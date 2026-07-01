#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, CanRx, Fifo, Id, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    eth::{self},
    peripherals::{self, CAN1},
    rng::{self},
    Config,
};
use embassy_time::{Duration, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    tasks::can::{
        board_heartbeat::send_heartbeat,
        send::{can_sender, CAN_SEND},
    },
};
use hyped_can::HypedCanFrame;
use hyped_communications::{
    boards::Board, data::CanData, measurements::MeasurementReading, messages::CanMessage,
};
use hyped_core::{
    config::MeasurementId,
    types::{Current, Temperature, Velocity, Voltage},
};
use panic_probe as _;

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
        .init(Board::TemperatureTester)
        .expect("Failed to initialize board simulator identity");

    let p = embassy_stm32::init(Config::default());

    defmt::info!("Board simulator starting");
    defmt::info!("Setting up CAN...");

    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();

    spawner.must_spawn(can_sender(can_tx));
    spawner.must_spawn(simulator_receiver(can_rx));
    spawner.must_spawn(send_heartbeat(Board::Telemetry));

    defmt::info!("Board simulator CAN setup complete");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn simulator_receiver(mut rx: CanRx<'static>) {
    loop {
        let envelope = rx.read().await;
        if envelope.is_err() {
            defmt::warn!("Simulator CAN receive error: {:?}", envelope.err());
            continue;
        }
        let envelope = envelope.unwrap();

        let id = envelope.frame.id();
        let raw_id = match id {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        };

        let mut data = [0u8; 8];
        data.copy_from_slice(envelope.frame.data());

        let frame = HypedCanFrame::new(raw_id, data);
        let message: CanMessage = frame.into();
        defmt::info!("Simulator received CAN message: {:?}", message);

        respond_to_message(message).await;
    }
}

async fn respond_to_message(message: CanMessage) {
    match message {
        CanMessage::Heartbeat(_heartbeat) => {
            // ignore for now
            defmt::debug!("Simulator received heartbeat");
        }
        CanMessage::StartPrechargeCommand => {
            defmt::info!("Simulator responding to StartPrechargeCommand");
            send_after(Duration::from_millis(2000), CanMessage::PrechargeStarted).await;
            send_after(
                Duration::from_millis(2000),
                CanMessage::ShutdownCircuitryRelayClosed,
            )
            .await;
            send_after(
                Duration::from_millis(2000),
                CanMessage::BatteryPrechargeRelayClosed,
            )
            .await;
            send_after(
                Duration::from_millis(2000),
                CanMessage::MotorControllerRelayClosed,
            )
            .await;
            send_after(Duration::from_millis(2000), CanMessage::PrechargeVoltageOK).await;
            send_after(Duration::from_millis(2000), CanMessage::PrechargeComplete).await;
        }
        CanMessage::OpenPrechargeRelaysCommand => {
            defmt::info!("Simulator responding to OpenPrechargeRelaysCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::BatteryPrechargeRelayOpen,
            )
            .await;
            send_after(
                Duration::from_millis(100),
                CanMessage::MotorControllerRelayOpen,
            )
            .await;
        }
        CanMessage::MotorControllerSetupCommand => {
            defmt::info!("Simulator responding to MotorControllerSetupCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::MotorControllerSetupComplete,
            )
            .await;
        }
        CanMessage::MotorControllerSetOperationalCommand => {
            defmt::info!("Simulator responding to MotorControllerSetOperationalCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::MotorControllerOperational,
            )
            .await;
        }
        CanMessage::UnclampBrakesCommand => {
            defmt::info!("Simulator responding to UnclampBrakesCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::BrakesUnclamped {
                    from: Board::Pneumatics,
                },
            )
            .await;
        }
        CanMessage::ClampBrakesCommand => {
            defmt::info!("Simulator responding to ClampBrakesCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::BrakesClamped {
                    from: Board::Pneumatics,
                },
            )
            .await;
        }
        CanMessage::StartPropulsionAccelerationCommand => {
            defmt::info!("Simulator responding to StartPropulsionAccelerationCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::PropulsionAccelerationStarted,
            )
            .await;
            send_after(Duration::from_millis(100), propulsion_status(0)).await;
            send_velocity_profile().await;
        }
        CanMessage::StartPropulsionBrakingCommand => {
            defmt::info!("Simulator responding to StartPropulsionBrakingCommand");
            send_after(
                Duration::from_millis(100),
                CanMessage::PropulsionBrakingStarted,
            )
            .await;
            send_after(Duration::from_millis(100), propulsion_status(0)).await;
        }
        CanMessage::StartDischargeCommand => {
            defmt::info!("Simulator responding to StartDischargeCommand");
            send_after(Duration::from_millis(100), CanMessage::DischargeStarted).await;
            send_after(Duration::from_millis(100), CanMessage::DischargeRelayClosed).await;
            send_after(
                Duration::from_millis(100),
                CanMessage::ShutdownCircuitryRelayOpen,
            )
            .await;
            send_after(Duration::from_millis(100), CanMessage::DischargeVoltageOK).await;
            send_after(Duration::from_millis(100), CanMessage::DischargeComplete).await;
        }
        _ => {}
    }
}

async fn send_after(delay: Duration, message: CanMessage) {
    Timer::after(delay).await;
    send_message(message).await;
}

async fn send_message(message: CanMessage) {
    defmt::info!("Simulator sending CAN message: {:?}", message);
    CAN_SEND.sender().send(message).await;
}

async fn send_velocity_profile() {
    let can_sender = CAN_SEND.sender();

    for velocity in 0u16..=10u16 {
        let message = CanMessage::MeasurementReading(MeasurementReading::new(
            CanData::U16(velocity),
            Board::Navigation,
            MeasurementId::Velocity,
        ));

        defmt::info!("Sending velocity reading {}", velocity);
        can_sender.send(message).await;

        if velocity < 10 {
            Timer::after(Duration::from_secs(1)).await;
        }
    }

    defmt::info!("Sending EndOfTrackBrake after velocity profile");
    can_sender.send(CanMessage::EndOfTrackBrake).await;
}

fn propulsion_status(velocity_kmh: u16) -> CanMessage {
    CanMessage::PropulsionStatus {
        current_ma: Current(0),
        velocity_kmh: Velocity(velocity_kmh),
        temperature_c: Temperature(25),
        voltage_cv: Voltage(0),
    }
}
