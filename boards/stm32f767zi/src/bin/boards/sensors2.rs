#![no_std]
#![no_main]

/// Sensors board 2
/// Uses PA1 and 2 for low pressure
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::{
    adc::{Adc, AdcChannel},
    bind_interrupts,
    can::{
        filter::Mask32, Can, CanRx, Fifo, Id, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    eth, gpio,
    peripherals::{self, ADC1, ADC2, CAN1},
    rng, Config,
};
use embassy_time::{Duration, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    io::Stm32f767ziAdc,
    tasks::can::{
        board_heartbeat::send_heartbeat,
        send::{can_sender, CAN_SEND},
    },
};
use hyped_communications::{boards::Board, messages::CanMessage};
use hyped_core::config::SENSORS_CONFIG;
use hyped_sensors::{low_pressure::LowPressure, SensorValueRange};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

/// The update frequency of the low pressure sensor.
const UPDATE_FREQUENCY: Duration = Duration::from_hz(10);
const PRECHARGE_SETTLE_TIME: Duration = Duration::from_secs(3);
const DISCHARGE_SETTLE_TIME: Duration = Duration::from_secs(30);

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD
        .init(Board::Sensors2)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    let gpio1: gpio::Output<'static> =
        gpio::Output::new(p.PF13, gpio::Level::Low, gpio::Speed::Low);
    let gpio2: gpio::Output<'static> = gpio::Output::new(p.PE9, gpio::Level::Low, gpio::Speed::Low);
    let gpio3: gpio::Output<'static> =
        gpio::Output::new(p.PE11, gpio::Level::Low, gpio::Speed::Low);
    let adc1 = Adc::new(p.ADC1);
    let pin1 = p.PA3.degrade_adc();

    let adc2 = Adc::new(p.ADC2);
    let pin2 = p.PA2.degrade_adc();

    let low_pressure_1 = LowPressure::new(Stm32f767ziAdc::new(
        adc1,
        pin1,
        SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    ));
    let low_pressure_2 = LowPressure::new(Stm32f767ziAdc::new(
        adc2,
        pin2,
        SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    ));

    let gpio_pins = Pins {
        shutdown_circuitry_relay: gpio1,
        battery_precharge_relay: gpio2,
        motor_controller_relay: gpio3,
    };
    let pressure_sensors = PressureSensors {
        low_pressure_1,
        low_pressure_2,
    };

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PB8, p.PB9, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_sender(can_tx));
    spawner.must_spawn(send_heartbeat(Board::Telemetry));
    spawner.must_spawn(sensors_board_can_receiver(can_rx, gpio_pins));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(sensors_board_pressure_sensors_task(pressure_sensors));

    loop {
        yield_now().await;
    }
}

struct Pins {
    shutdown_circuitry_relay: gpio::Output<'static>,
    battery_precharge_relay: gpio::Output<'static>,
    motor_controller_relay: gpio::Output<'static>,
}

struct PressureSensors {
    low_pressure_1: LowPressure<Stm32f767ziAdc<'static, ADC1>>,
    low_pressure_2: LowPressure<Stm32f767ziAdc<'static, ADC2>>,
}

#[embassy_executor::task]
async fn sensors_board_pressure_sensors_task(mut pressure_sensors: PressureSensors) {
    loop {
        let low_pressures_ok = [
            !matches!(
                pressure_sensors.low_pressure_1.read_pressure(),
                Some(SensorValueRange::Critical(_))
            ),
            !matches!(
                pressure_sensors.low_pressure_2.read_pressure(),
                Some(SensorValueRange::Critical(_))
            ),
        ]
        .iter()
        .all(|b| *b);

        if !low_pressures_ok {
            defmt::warn!("Pressure sensor out of safe range, sending emergency");
            CAN_SEND
                .send(CanMessage::Emergency(
                    Board::Sensors2,
                    hyped_communications::events::Reason::Pressure,
                ))
                .await;
            return;
        }

        Timer::after(UPDATE_FREQUENCY).await;
    }
}

#[embassy_executor::task]
async fn sensors_board_can_receiver(mut rx: CanRx<'static>, mut gpio_pins: Pins) {
    loop {
        defmt::debug!("Waiting for CAN message");

        let envelope = rx.read().await;
        if envelope.is_err() {
            defmt::warn!("CAN receive error: {:?}", envelope.err());
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

        let frame = hyped_can::HypedCanFrame::new(raw_id, data);
        let message: CanMessage = frame.into();
        defmt::info!("Received CAN message: {:?}", message);

        respond_to_message(message, &mut gpio_pins).await;
    }
}

async fn respond_to_message(message: CanMessage, gpio_pins: &mut Pins) {
    match message {
        CanMessage::Heartbeat(_heartbeat) => {
            defmt::debug!("Heartbeat received");
        }
        CanMessage::StartPrechargeCommand => {
            defmt::info!("StartPrechargeCommand received");
            CAN_SEND.send(CanMessage::PrechargeStarted).await;

            gpio_pins.close_shutdown_circuitry_relay().await;
            gpio_pins.close_battery_precharge_relay().await;
            Timer::after(PRECHARGE_SETTLE_TIME).await;
            CAN_SEND.send(CanMessage::PrechargeVoltageOK).await;
            gpio_pins.close_motor_controller_relay().await;
            CAN_SEND.send(CanMessage::PrechargeComplete).await;
        }
        CanMessage::OpenPrechargeRelaysCommand => {
            defmt::info!("OpenPrechargeRelaysCommand received");
            gpio_pins.open_precharge_relays().await;
        }
        CanMessage::StartDischargeCommand => {
            defmt::info!("StartDischargeCommand received");
            CAN_SEND.send(CanMessage::DischargeStarted).await;
            gpio_pins.open_shutdown_circuitry_relay().await;
            Timer::after(DISCHARGE_SETTLE_TIME).await;
            CAN_SEND.send(CanMessage::DischargeVoltageOK).await;
            CAN_SEND.send(CanMessage::DischargeComplete).await;
        }
        CanMessage::UnclampBrakesCommand => {
            defmt::info!("UnclampBrakesCommand received");
            CAN_SEND
                .send(CanMessage::BrakesUnclamped {
                    from: Board::Sensors2,
                })
                .await;
        }
        CanMessage::ClampBrakesCommand => {
            defmt::info!("ClampBrakesCommand received");
            CAN_SEND
                .send(CanMessage::BrakesClamped {
                    from: Board::Sensors2,
                })
                .await;
        }
        CanMessage::Emergency(from, reason) => {
            defmt::warn!("EMERGENCY: from {:?} reason={}", from, reason);
            gpio_pins.open_shutdown_circuitry_relay().await;
        }
        _ => {
            defmt::debug!("Ignored CAN message: {:?}", message);
        }
    }
}

impl Pins {
    async fn open_precharge_relays(&mut self) {
        self.battery_precharge_relay.set_low();
        CAN_SEND.send(CanMessage::BatteryPrechargeRelayOpen).await;

        self.motor_controller_relay.set_low();
        CAN_SEND.send(CanMessage::MotorControllerRelayOpen).await;
    }

    async fn close_battery_precharge_relay(&mut self) {
        self.battery_precharge_relay.set_high();
        CAN_SEND.send(CanMessage::BatteryPrechargeRelayClosed).await;
    }

    async fn close_motor_controller_relay(&mut self) {
        self.motor_controller_relay.set_high();
        CAN_SEND.send(CanMessage::MotorControllerRelayClosed).await;
    }

    async fn close_shutdown_circuitry_relay(&mut self) {
        self.shutdown_circuitry_relay.set_high();
        CAN_SEND
            .send(CanMessage::ShutdownCircuitryRelayClosed)
            .await;
    }

    async fn open_shutdown_circuitry_relay(&mut self) {
        self.shutdown_circuitry_relay.set_low();
        CAN_SEND.send(CanMessage::ShutdownCircuitryRelayOpen).await;
    }
}
