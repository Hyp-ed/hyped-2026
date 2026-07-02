#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    gpio::{Input, Level, Output, Pull, Speed},
    i2c::I2c,
    init,
    mode::Blocking,
    peripherals::CAN1,
    spi::{self, BitOrder, Spi},
    time::{khz, Hertz},
};
use embassy_sync::{
    blocking_mutex::{
        raw::{CriticalSectionRawMutex, NoopRawMutex},
        Mutex,
    },
    watch::Watch,
};
use embassy_time::{Duration, Timer};
use heapless::Vec;
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    io::{Stm32f767ziGpioOutput, Stm32f767ziSpi},
    tasks::{
        can::send::{can_sender, CAN_SEND},
        sensors::{
            read_accelerometers_from_mux::{
                read_accelerometers_from_mux, AccelerometerMuxReadings,
            },
            read_keyence::read_keyence,
            read_optical_flow::read_optical_flow,
        },
    },
};
use hyped_communications::{
    boards::Board, data::CanData, measurements::MeasurementReading, messages::CanMessage,
};
use hyped_core::config::{MeasurementId, LOCALISATION_CONFIG};
use hyped_localisation::{control::localizer::Localizer, types::RawAccelerometerData};
use hyped_spi::HypedSpiCsPin;
use panic_probe as _;
use static_cell::StaticCell;
type I2c1Bus = Mutex<NoopRawMutex, RefCell<I2c<'static, Blocking>>>;

/// A Watch to hold the latest Keyence stripe count
static KEYENCE_1_STRIPE_COUNT: Watch<CriticalSectionRawMutex, u32, 1> = Watch::new();
static KEYENCE_2_STRIPE_COUNT: Watch<CriticalSectionRawMutex, u32, 1> = Watch::new();

/// A Watch to hold the latest optical flow data
static OPTICAL_FLOW_DATA: Watch<CriticalSectionRawMutex, Vec<f64, 2>, 1> = Watch::new();

/// A Watch to hold the latest accelerometer data
static ACCELEROMETERS_DATA: Watch<CriticalSectionRawMutex, AccelerometerMuxReadings, 1> =
    Watch::new();

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

const TRACK_LENGTH_M: f64 = 100.0;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Import `init` so that we can initialize board peripherals.
    let p = init(Default::default());

    THIS_BOARD
        .init(Board::Navigation)
        .expect("Failed to initialize board identity");
    defmt::info!("Board identity initialised");

    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, _) = can.split();
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN sender task started");

    let can_message_sender = CAN_SEND.sender();

    let board = *THIS_BOARD.get().await;
    let mut end_of_track_brake_sent = false;

    let mut spi_config = spi::Config::default();
    spi_config.frequency = khz(400);
    spi_config.bit_order = BitOrder::MsbFirst;

    let spi = Spi::new_blocking(p.SPI1, p.PB3, p.PB5, p.PB4, spi_config);
    let hyped_spi = Stm32f767ziSpi::new(spi);

    let cs = HypedSpiCsPin::new(Stm32f767ziGpioOutput::new(Output::new(
        p.PA4,
        Level::High,
        Speed::VeryHigh,
    )));

    let i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz(200_000), Default::default());

    // Initialize the I2C bus and store it in a static cell so that it can be accessed from the task.
    static I2C_BUS: StaticCell<I2c1Bus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(RefCell::new(i2c)));
    defmt::info!("I2C initialized.");

    spawner
        .spawn(read_optical_flow(hyped_spi, cs, OPTICAL_FLOW_DATA.sender()))
        .unwrap();

    spawner
        .spawn(read_keyence(
            Input::new(p.PC13, Pull::Down),
            MeasurementId::Keyence1,
            KEYENCE_1_STRIPE_COUNT.sender(),
        ))
        .unwrap();
    spawner
        .spawn(read_keyence(
            Input::new(p.PC14, Pull::Down),
            MeasurementId::Keyence2,
            KEYENCE_2_STRIPE_COUNT.sender(),
        ))
        .unwrap();

    spawner
        .spawn(read_accelerometers_from_mux(
            i2c_bus,
            ACCELEROMETERS_DATA.sender(),
        ))
        .unwrap();

    // Initialise receivers
    let mut keyence_1_receiver = KEYENCE_1_STRIPE_COUNT.receiver().unwrap();
    let mut keyence_2_receiver = KEYENCE_2_STRIPE_COUNT.receiver().unwrap();
    let mut optical_flow_receiver = OPTICAL_FLOW_DATA.receiver().unwrap();
    let mut accelerometers_receiver = ACCELEROMETERS_DATA.receiver().unwrap();

    let mut localizer = Localizer::new();

    info!("Starting localizer loop...");

    loop {
        let keyence_data: Vec<u32, 2> = Vec::from_slice(&[
            keyence_1_receiver.get().await,
            keyence_2_receiver.get().await,
        ])
        .unwrap();

        let accelerometer_data: RawAccelerometerData<
            { LOCALISATION_CONFIG.accelerometers.num_sensors as usize },
            { LOCALISATION_CONFIG.num_axis as usize },
        > = accelerometers_receiver.get().await;

        let optical_data = optical_flow_receiver.get().await;

        match localizer.iteration(optical_data, keyence_data, accelerometer_data) {
            Ok(()) => {
                defmt::info!(
                    "Iteration OK: displacement = {} m, velocity = {} m/s, acceleration = {} m/s**2",
                    localizer.displacement,
                    localizer.velocity,
                    localizer.acceleration
                );

                can_message_sender
                    .send(CanMessage::MeasurementReading(MeasurementReading::new(
                        CanData::F32(localizer.displacement as f32),
                        board,
                        MeasurementId::Displacement,
                    )))
                    .await;
                can_message_sender
                    .send(CanMessage::MeasurementReading(MeasurementReading::new(
                        CanData::F32(localizer.velocity as f32),
                        board,
                        MeasurementId::Velocity,
                    )))
                    .await;
                can_message_sender
                    .send(CanMessage::MeasurementReading(MeasurementReading::new(
                        CanData::F32(localizer.acceleration as f32),
                        board,
                        MeasurementId::Acceleration,
                    )))
                    .await;

                //For demonstration purposes just brake when we're halfway down the track
                //In reality we would calculate the braking point based on localiser data
                if !end_of_track_brake_sent && localizer.displacement >= TRACK_LENGTH_M / 2.0 {
                    defmt::info!(
                        "Halfway to track end reached ({} m) — sending EndOfTrackBrake",
                        localizer.displacement
                    );
                    can_message_sender.send(CanMessage::EndOfTrackBrake).await;
                    end_of_track_brake_sent = true;
                }
            }
            Err(e) => {
                defmt::error!("Iteration error: {:?}", e);
            }
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
