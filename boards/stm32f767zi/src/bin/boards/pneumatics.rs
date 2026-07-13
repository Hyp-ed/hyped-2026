#![no_std]
#![no_main]

/// Pneumatics board
/// Uses PC12 and PC13 for high pressure
/// Uses PA1, 2, and 3 for low pressure
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, CanRx, Fifo, Id, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    eth, gpio,
    peripherals::{self, CAN1},
    rng, Config,
};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    tasks::can::{
        board_heartbeat::send_heartbeat,
        send::{can_sender, CAN_SEND},
    },
};
use hyped_communications::{boards::Board, messages::CanMessage};

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
        .init(Board::Pneumatics)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    // High pressure sensor (digital GPIO inputs)
    // let gpio1: Stm32f767ziGpioInput =
    //     Stm32f767ziGpioInput::new(gpio::Input::new(p.PC12, gpio::Pull::Down));
    // let gpio2: Stm32f767ziGpioInput =
    //     Stm32f767ziGpioInput::new(gpio::Input::new(p.PC13, gpio::Pull::Down));
    // let high_pressure = HighPressure::new(gpio1, gpio2);

    // Low pressure sensors on ADC1, ADC2, ADC3
    // let adc1 = Adc::new(p.ADC1);
    // let pin1 = p.PA3.degrade_adc();

    // let adc2 = Adc::new(p.ADC2);
    // let pin2 = p.PA2.degrade_adc();

    // let adc3 = Adc::new(p.ADC3);
    // let pin3 = p.PA1.degrade_adc();

    // let low_pressure_1 = LowPressure::new(Stm32f767ziAdc::new(
    //     adc1,
    //     pin1,
    //     SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    // ));
    // let low_pressure_2 = LowPressure::new(Stm32f767ziAdc::new(
    //     adc2,
    //     pin2,
    //     SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    // ));
    // let low_pressure_3 = LowPressure::new(Stm32f767ziAdc::new(
    //     adc3,
    //     pin3,
    //     SENSORS_CONFIG.sensors.low_pressure.v_ref as f32,
    // ));
    let brake_gpio = gpio::Output::new(p.PF15, gpio::Level::Low, gpio::Speed::Low);

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_sender(can_tx));
    spawner.must_spawn(send_heartbeat(Board::Telemetry));
    spawner.must_spawn(pneumatics_can_receiver(can_rx, brake_gpio));
    defmt::info!("CAN setup complete");

    // let pressure_sensors = PressureSensors {
    //     high_pressure,
    //     low_pressure_1,
    //     low_pressure_2,
    //     low_pressure_3,
    // };

    // spawner.must_spawn(pneumatics_response_task(pressure_sensors));

    loop {
        yield_now().await;
    }
}

// struct PressureSensors {
//     high_pressure: HighPressure<Stm32f767ziGpioInput>,
//     low_pressure_1: LowPressure<Stm32f767ziAdc<'static, ADC1>>,
//     low_pressure_2: LowPressure<Stm32f767ziAdc<'static, ADC2>>,
//     low_pressure_3: LowPressure<Stm32f767ziAdc<'static, ADC3>>,
// }

#[embassy_executor::task]
async fn pneumatics_can_receiver(mut rx: CanRx<'static>, mut brake_gpio: gpio::Output<'static>) {
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
        //defmt::info!("Received CAN message: {:?}", message);

        respond_to_message(message, &mut brake_gpio).await;
    }
}

async fn respond_to_message(message: CanMessage, brake_gpio: &mut gpio::Output<'static>) {
    match message {
        CanMessage::Heartbeat(_heartbeat) => {
            defmt::debug!("Heartbeat received");
        }
        CanMessage::UnclampBrakesCommand => {
            defmt::info!("UnclampBrakesCommand received");
            brake_gpio.set_high();
            CAN_SEND
                .send(CanMessage::BrakesUnclamped {
                    from: Board::Pneumatics,
                })
                .await;
        }
        CanMessage::ClampBrakesCommand => {
            defmt::info!("ClampBrakesCommand received");
            brake_gpio.set_low();
            defmt::info!("Brakes clamped, sending BrakesClamped message");
            CAN_SEND
                .send(CanMessage::BrakesClamped {
                    from: Board::Pneumatics,
                })
                .await;
        }
        CanMessage::Emergency(from, reason) => {
            defmt::warn!("EMERGENCY: from {:?} reason={}", from, reason);
            brake_gpio.set_low();
        }
        _ => {
            defmt::debug!("Ignored CAN message: {:?}", message);
        }
    }
}

// #[embassy_executor::task]
// async fn pneumatics_response_task(mut pressure_sensors: PressureSensors) {
//     loop {
//         // Read high pressure sensor
//         let high_pressure_ok = matches!(
//             pressure_sensors.high_pressure.get_high_pressure_state(),
//             Ok(high_pressure::State::LowRange)
//         );
//
//         // Read all three low pressure sensors
//         let low_pressures_ok = [
//             !matches!(
//                 pressure_sensors.low_pressure_1.read_pressure(),
//                 Some(SensorValueRange::Critical(_))
//             ),
//             !matches!(
//                 pressure_sensors.low_pressure_2.read_pressure(),
//                 Some(SensorValueRange::Critical(_))
//             ),
//             !matches!(
//                 pressure_sensors.low_pressure_3.read_pressure(),
//                 Some(SensorValueRange::Critical(_))
//             ),
//         ]
//         .iter()
//         .all(|b| *b);
//
//         if !high_pressure_ok || !low_pressures_ok {
//             defmt::warn!("Pressure sensor out of safe range, sending emergency");
//
//             CAN_SEND
//                 .send(CanMessage::Emergency(
//                     Board::Pneumatics,
//                     hyped_communications::emergency::Reason::Pressure,
//                 ))
//                 .await;
//
//             return;
//         }
//
//         Timer::after(UPDATE_FREQUENCY).await;
//     }
// }
