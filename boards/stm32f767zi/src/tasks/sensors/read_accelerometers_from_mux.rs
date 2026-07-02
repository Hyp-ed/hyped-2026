use crate::io::Stm32f767ziI2c;
use core::cell::RefCell;
use defmt_rtt as _;
use embassy_stm32::{i2c::I2c, mode::Blocking};
use embassy_sync::{
    blocking_mutex::{
        raw::{CriticalSectionRawMutex, NoopRawMutex},
        Mutex,
    },
    watch::Sender,
};
use embassy_time::{Duration, Timer};
use heapless::Vec;
use hyped_core::config::SENSORS_CONFIG;
use hyped_i2c::{i2c_mux::DEFAULT_MUX_ADDRESS, HypedI2c};
use hyped_localisation::{
    config::{NUM_ACCELEROMETERS, NUM_AXIS},
    types::RawAccelerometerData,
};
use hyped_sensors::accelerometer::{self, Accelerometer, AccelerometerAddresses};

pub type AccelerometerMuxReadings = RawAccelerometerData<NUM_ACCELEROMETERS, NUM_AXIS>;

type I2c1Bus = Mutex<NoopRawMutex, RefCell<I2c<'static, Blocking>>>;

const WORKING_ACCELEROMETER_MUX_CHANNEL: u8 = 1;
const WORKING_ACCELEROMETER_ADDRESS: AccelerometerAddresses = AccelerometerAddresses::Address1e;

/// Task that reads the accelerometers on the muxes given in `mux_address_channel_pairs`
#[embassy_executor::task]
pub async fn read_accelerometers_from_mux(
    i2c_bus: &'static I2c1Bus,
    sender: Sender<'static, CriticalSectionRawMutex, AccelerometerMuxReadings, 1>,
) -> ! {
    let mut hyped_i2c = Stm32f767ziI2c::new(i2c_bus);

    // Select the mux channel with the known working accelerometer.
    hyped_i2c
        .write_byte(DEFAULT_MUX_ADDRESS, 1 << WORKING_ACCELEROMETER_MUX_CHANNEL)
        .expect("Failed to select working accelerometer channel on mux");
    defmt::info!(
        "Mux initialized and channel {} selected.",
        WORKING_ACCELEROMETER_MUX_CHANNEL
    );

    let mut i2c_for_accelerometer_1 = Stm32f767ziI2c::new(i2c_bus);
    let mut accelerometer_1 =
        Accelerometer::new(&mut i2c_for_accelerometer_1, WORKING_ACCELEROMETER_ADDRESS)
            .expect("Failed to create accelerometer. Check the wiring and the I2C address of the sensor.");
    defmt::info!("Accelerometer 1 initialized.");

    loop {
        defmt::info!("Reading accelerometer from mux");

        // Reselect the mux channel before each read in case another task touched the mux.
        hyped_i2c
            .write_byte(DEFAULT_MUX_ADDRESS, 1 << WORKING_ACCELEROMETER_MUX_CHANNEL)
            .expect("Failed to select working accelerometer channel on mux");

        // Read the first accelerometer
        match accelerometer_1.check_status() {
            accelerometer::Status::Ok => {}
            accelerometer::Status::DataNotReady => {
                defmt::warn!("Accelerometer is not ready to provide data")
            }
            accelerometer::Status::Unknown => {
                panic!("Could not get status of accelerometer")
            }
        }
        let reading_1 = accelerometer_1.read().unwrap();

        let readings: RawAccelerometerData<NUM_ACCELEROMETERS, NUM_AXIS> =
            Vec::from_slice(&[reading_1]).unwrap();

        sender.send(readings);
        Timer::after(Duration::from_hz(
            SENSORS_CONFIG.sensors.accelerometer.update_frequency as u64,
        ))
        .await;
    }
}
