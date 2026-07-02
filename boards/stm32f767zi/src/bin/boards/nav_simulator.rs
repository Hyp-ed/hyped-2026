#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    init,
    peripherals::CAN1,
};
use embassy_time::{Duration, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    tasks::can::{
        board_heartbeat::send_heartbeat,
        receive::can_receiver,
        send::{can_sender, CAN_SEND},
    },
};
use hyped_communications::{
    boards::Board, bus, bus::DynSubscriber, events::Event, messages::CanMessage,
};
use hyped_core::types::{Current, Temperature, Velocity, Voltage};
use panic_probe as _;

const BRAKE_DELAY_S: u64 = 2;

static NAV_SIM_ARMED: AtomicBool = AtomicBool::new(false);
static NAV_SIM_GENERATION: AtomicU32 = AtomicU32::new(0);

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = init(Default::default());

    THIS_BOARD
        .init(Board::Navigation)
        .expect("Failed to initialize nav simulator identity");
    bus::init().expect("Failed to initialise event bus");
    let navigation_events =
        bus::subscriber().expect("Failed to create navigation event subscriber");

    info!("Navigation simulator starting");

    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();

    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    spawner.must_spawn(send_heartbeat(Board::Telemetry));
    spawner.must_spawn(navigation_simulator_task(spawner, navigation_events));

    info!(
        "Navigation simulator armed to brake {}s after acceleration command",
        BRAKE_DELAY_S
    );

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn navigation_simulator_task(
    spawner: Spawner,
    mut events: DynSubscriber<'static, Event>,
) {
    loop {
        match events.next_message_pure().await {
            Event::StartPropulsionAccelerationCommand => {
                let generation = NAV_SIM_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
                NAV_SIM_ARMED.store(true, Ordering::Release);
                info!(
                    "Navigation simulator armed; braking in {}s",
                    BRAKE_DELAY_S
                );

                if spawner.spawn(send_delayed_brake(generation)).is_err() {
                    error!("Failed to spawn delayed brake task");
                }
            }
            Event::StartPropulsionBrakingCommand | Event::EndOfTrackBrakeCommand => {
                NAV_SIM_ARMED.store(false, Ordering::Release);
                NAV_SIM_GENERATION.fetch_add(1, Ordering::AcqRel);
                info!("Navigation simulator disarmed");
                let _ = spawner.spawn(send_stopped_status());
            }
            _ => {}
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn send_stopped_status() {
    Timer::after(Duration::from_secs(2)).await;
    info!("Navigation simulator reporting pod stopped");
    CAN_SEND
        .sender()
        .send(CanMessage::PropulsionStatus {
            current_ma: Current(0),
            velocity_kmh: Velocity(0),
            temperature_c: Temperature(0),
            voltage_cv: Voltage(0),
        })
        .await;
}

#[embassy_executor::task(pool_size = 2)]
async fn send_delayed_brake(generation: u32) {
    Timer::after(Duration::from_secs(BRAKE_DELAY_S)).await;

    if NAV_SIM_ARMED.load(Ordering::Acquire)
        && NAV_SIM_GENERATION.load(Ordering::Acquire) == generation
    {
        info!("Navigation simulator sending EndOfTrackBrake");
        CAN_SEND.sender().send(CanMessage::EndOfTrackBrake).await;
        NAV_SIM_ARMED.store(false, Ordering::Release);
    } else {
        info!("Navigation simulator delayed brake cancelled");
    }
}
