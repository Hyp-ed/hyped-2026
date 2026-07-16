#![no_std]
#![no_main]

use core::str::FromStr;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    eth::{self, generic_smi::GenericSMI, Ethernet, PacketQueue},
    peripherals::{self, CAN1, ETH},
    rng::{self, Rng},
    time::Hertz,
    Config,
};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    configure_networking, default_can_config,
    log::log,
    set_up_network_stack,
    tasks::{
        can::{
            board_heartbeat::send_heartbeat,
            event_to_can::event_to_can,
            receive::{can_receiver, INCOMING_HEARTBEATS},
            send::{can_sender, CAN_SEND},
        },
        can_to_mqtt::can_to_mqtt,
        mqtt::{
            base_station_heartbeat::base_station_heartbeat, mqtt,
            mqtt_to_event_bus::mqtt_to_event_bus,
        },
        network::net_task,
        sensors::read_imd,
    },
};
use hyped_communications::{boards::Board, bus, emergency::Reason, messages::CanMessage};
use hyped_core::{
    config::{HEARTBEAT_CONFIG, TELEMETRY_CONFIG},
    log_types::LogLevel,
};
use hyped_state_machine::state_machine::{run, StateMachine};
use panic_probe as _;
use rand_core::RngCore;
use static_cell::StaticCell;

const HEARTBEAT_BOARDS: [Board; 4] = [
    Board::Navigation,
    Board::MotorControl,
    Board::Pneumatics,
    Board::HighPower,
];

//const HEARTBEAT_BOARDS: [Board; 1] = [Board::TemperatureTester];

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD
        .init(Board::Telemetry)
        .expect("Failed to initialize board");

    let mut config = Config::default();
    configure_networking!(config);
    let p = embassy_stm32::init(config);
    set_up_network_stack!(p, stack, spawner);

    // Network tasks: MQTT and base station heartbeat
    spawner.must_spawn(mqtt(stack));
    Timer::after(Duration::from_secs(2)).await;
    spawner.must_spawn(base_station_heartbeat());
    defmt::info!("Base station heartbeat task started");

    bus::init().expect("Failed to initialise event bus publisher");
    let state_machine_events =
        bus::subscriber().expect("Failed to create state machine subscriber");
    let can_bridge_events = bus::subscriber().expect("Failed to create CAN bridge subscriber");

    // CAN tasks: CAN send/receive, heartbeat controller, and state machine
    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(can_to_mqtt());
    for board in HEARTBEAT_BOARDS {
        spawner.must_spawn(send_heartbeat(board));
    }
    spawner.must_spawn(heartbeat_monitor());
    spawner.must_spawn(read_imd::read_imd());
    spawner.must_spawn(mqtt_to_event_bus());
    spawner.must_spawn(event_to_can(can_bridge_events));
    // Let the CAN bridge start listening before the state machine entry publishes commands.
    Timer::after(Duration::from_millis(10)).await;
    // ... add more boards here
    spawner.must_spawn(run(StateMachine::new(), state_machine_events));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn heartbeat_monitor() {
    let can_sender = CAN_SEND.sender();
    let mut seen = [false; HEARTBEAT_BOARDS.len()];
    let mut last_seen = [Instant::now(); HEARTBEAT_BOARDS.len()];

    let first_heartbeat_result = with_timeout(
        Duration::from_secs(HEARTBEAT_CONFIG.boards.startup_timeout_s as u64),
        async {
            while !seen.iter().all(|board_seen| *board_seen) {
                let heartbeat = INCOMING_HEARTBEATS.receive().await;
                if heartbeat.to != Board::Telemetry {
                    continue;
                }
                if let Some(index) = heartbeat_board_index(heartbeat.from) {
                    seen[index] = true;
                    last_seen[index] = Instant::now();
                    defmt::info!("Initial heartbeat received from {:?}", heartbeat.from);
                }
            }
        },
    )
    .await;

    if first_heartbeat_result.is_err() {
        for (index, board) in HEARTBEAT_BOARDS.iter().enumerate() {
            if !seen[index] {
                defmt::error!("No initial heartbeat from {:?}", board);
            }
        }
        can_sender
            .send(CanMessage::Emergency(
                Board::Telemetry,
                Reason::NoInitialHeartbeat,
            ))
            .await;
        bus::publish(hyped_communications::events::Event::Emergency {
            from: Board::Telemetry,
            reason: Reason::NoInitialHeartbeat,
        })
        .await;
        return;
    }

    loop {
        if let Ok(heartbeat) = with_timeout(
            Duration::from_millis(HEARTBEAT_CONFIG.boards.max_latency_ms as u64),
            INCOMING_HEARTBEATS.receive(),
        )
        .await
        {
            if heartbeat.to == Board::Telemetry {
                if let Some(index) = heartbeat_board_index(heartbeat.from) {
                    last_seen[index] = Instant::now();
                }
            }
        }

        let now = Instant::now();
        for (index, board) in HEARTBEAT_BOARDS.iter().enumerate() {
            if now.duration_since(last_seen[index])
                > Duration::from_millis(HEARTBEAT_CONFIG.boards.max_latency_ms as u64)
            {
                defmt::error!("Missing heartbeat from {:?}", board);
                can_sender
                    .send(CanMessage::Emergency(
                        Board::Telemetry,
                        Reason::MissingHeartbeat,
                    ))
                    .await;
                bus::publish(hyped_communications::events::Event::Emergency {
                    from: Board::Telemetry,
                    reason: Reason::MissingHeartbeat,
                })
                .await;
                return;
            }
        }
    }
}

fn heartbeat_board_index(board: Board) -> Option<usize> {
    HEARTBEAT_BOARDS
        .iter()
        .position(|heartbeat_board| *heartbeat_board == board)
}
