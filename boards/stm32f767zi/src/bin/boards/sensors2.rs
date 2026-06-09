#![no_std]
#![no_main]

/// Sensors board 2
/// Uses PA1 and 2 for low pressure
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::{
    adc::{Adc, AdcChannel},
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
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
        receive::can_receiver,
        send::{can_sender, CAN_SEND},
    },
};
use hyped_communications::{
    boards::Board,
    bus::{self, EVENT_BUS},
    events::Event,
    messages::CanMessage,
};
use hyped_core::config::SENSORS_CONFIG;
use hyped_sensors::{low_pressure::LowPressure, SensorValueRange};

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

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD
        .init(Board::Sensors1)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    let gpio1: gpio::Output<'static> =
        gpio::Output::new(p.PF13, gpio::Level::High, gpio::Speed::Low);
    let gpio2: gpio::Output<'static> =
        gpio::Output::new(p.PE9, gpio::Level::High, gpio::Speed::Low);
    let gpio3: gpio::Output<'static> =
        gpio::Output::new(p.PE11, gpio::Level::High, gpio::Speed::Low);
    let gpio4: gpio::Output<'static> =
        gpio::Output::new(p.PF14, gpio::Level::High, gpio::Speed::Low);

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
        gpio4,
    };
    let pressure_sensors = PressureSensors {
        low_pressure_1,
        low_pressure_2,
    };

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(sensors_board_response_task(gpio_pins));
    spawner.must_spawn(sensors_board_pressure_sensors_task(pressure_sensors));

    loop {
        yield_now().await;
    }
}

struct Pins {
    shutdown_circuitry_relay: gpio::Output<'static>,
    battery_precharge_relay: gpio::Output<'static>,
    motor_controller_relay: gpio::Output<'static>,
    gpio4: gpio::Output<'static>,
}

struct PressureSensors {
    low_pressure_1: LowPressure<Stm32f767ziAdc<'static, ADC1>>,
    low_pressure_2: LowPressure<Stm32f767ziAdc<'static, ADC2>>,
}

#[embassy_executor::task]
async fn sensors_board_pressure_sensors_task(mut pressure_sensors: PressureSensors) {
    loop {
        // Read all three low pressure sensors
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
async fn sensors_board_response_task(mut gpio_pins: Pins) {
    bus::init().expect("Failed to init sensors board 2 event bus");

    let mut sub = EVENT_BUS
        .subscriber()
        .expect("Failed to run sensors board 2 state machine");

    use embassy_sync_compat::pubsub::WaitResult;

    loop {
        let event = sub.next_message().await;

        match event {
            WaitResult::Message(Event::StartPrechargeCommand) => {
                CAN_SEND.send(CanMessage::PrechargeStarted).await;

                gpio_pins.shutdown_circuitry_relay.set_high();
                CAN_SEND
                    .send(CanMessage::ShutdownCircuitryRelayClosed)
                    .await;
                gpio_pins.gpio4.set_high();

                gpio_pins.battery_precharge_relay.set_low();

                Timer::after_secs(4).await;
                gpio_pins.gpio4.set_low();

                gpio_pins.battery_precharge_relay.set_high();
                CAN_SEND.send(CanMessage::BatteryPrechargeRelayClosed).await;

                Timer::after_secs(2).await;

                CAN_SEND.send(CanMessage::PrechargeVoltageOK).await;

                gpio_pins.motor_controller_relay.set_high();

                CAN_SEND.send(CanMessage::MotorControllerRelayClosed).await;

                // there is a possibility that after 20s of this relay being on it needs to be turned back off, please have code for this commented for now until further confirmation
                // Timer::after_secs(20).await;
                //
                // gpio_pins.gpio2.set_low();
                // gpio_pins.gpio3.set_low();

                CAN_SEND.send(CanMessage::PrechargeComplete).await;
            }

            WaitResult::Message(Event::Emergency { from, reason }) => {
                defmt::warn!("EMERGENCY: from {:?} reason={}", from, reason);
                return;
            }

            WaitResult::Message(Event::Heartbeat { from }) => {
                defmt::debug!("Heartbeat from {:?}", from);
            }

            _ => {} // Ignore other events
        }
    }
}
