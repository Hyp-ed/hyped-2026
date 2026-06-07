#![no_std]
#![no_main]

/// Sensors board 1
/// Uses PC12 and PC13 for high pressure
/// Uses PA1, 2, and 3 for low pressure
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
    peripherals::{self, ADC1, ADC2, ADC3, CAN1},
    rng, Config,
};
use embassy_time::{Duration, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    io::{Stm32f767ziAdc, Stm32f767ziGpioInput},
    tasks::can::{receive::can_receiver, send::can_sender},
};
use hyped_communications::{boards::Board, bus::EVENT_BUS, events::Event};
use hyped_core::config::SENSORS_CONFIG;
use hyped_sensors::{
    high_pressure::{self, HighPressure},
    low_pressure::LowPressure,
    SensorValueRange,
};

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
        .init(Board::Sensors2)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    // High pressure sensor (digital GPIO inputs)
    let gpio1: Stm32f767ziGpioInput =
        Stm32f767ziGpioInput::new(gpio::Input::new(p.PC12, gpio::Pull::Down));
    let gpio2: Stm32f767ziGpioInput =
        Stm32f767ziGpioInput::new(gpio::Input::new(p.PC13, gpio::Pull::Down));
    let high_pressure = HighPressure::new(gpio1, gpio2);

    // Low pressure sensors on ADC1, ADC2, ADC3
    let adc1 = Adc::new(p.ADC1);
    let pin1 = p.PA3.degrade_adc();

    let adc2 = Adc::new(p.ADC2);
    let pin2 = p.PA2.degrade_adc();

    let adc3 = Adc::new(p.ADC3);
    let pin3 = p.PA1.degrade_adc();

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
    let low_pressure_3 = LowPressure::new(Stm32f767ziAdc::new(
        adc3,
        pin3,
        SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    ));

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN setup complete");

    let pressure_sensors = PressureSensors {
        high_pressure,
        low_pressure_1: low_pressure_1,
        low_pressure_2: low_pressure_2,
        low_pressure_3: low_pressure_3,
    };

    spawner.must_spawn(sensors_board_response_task(pressure_sensors));

    loop {
        yield_now().await;
    }
}

struct PressureSensors {
    high_pressure: HighPressure<Stm32f767ziGpioInput>,
    low_pressure_1: LowPressure<Stm32f767ziAdc<'static, ADC1>>,
    low_pressure_2: LowPressure<Stm32f767ziAdc<'static, ADC2>>,
    low_pressure_3: LowPressure<Stm32f767ziAdc<'static, ADC3>>,
}

#[embassy_executor::task]
async fn sensors_board_response_task(mut pressure_sensors: PressureSensors) {
    loop {
        // Read high pressure sensor
        let high_pressure_ok = matches!(
            pressure_sensors.high_pressure.get_high_pressure_state(),
            Ok(high_pressure::State::LowRange)
        );

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
            !matches!(
                pressure_sensors.low_pressure_3.read_pressure(),
                Some(SensorValueRange::Critical(_))
            ),
        ]
        .iter()
        .all(|b| *b);

        if !high_pressure_ok || !low_pressures_ok {
            defmt::warn!("Pressure sensor out of safe range, sending emergency");
            EVENT_BUS
                .sender()
                .send(Event::Emergency {
                    from: Board::Sensors2,
                    reason: hyped_communications::events::Reason::Pressure,
                })
                .await;
            return;
        }

        Timer::after(UPDATE_FREQUENCY).await;
    }
}
