use hyped_sensors::led_driver::{LedDriver, LedBank, LedChannel, LedDriverError}
use defmt_rtt as _;
use embassy_stm32::I2c;
use embassy_exeuctor::Spawner;

#[embassy_executor::main]

//this is a led driver test run
async fn main(spawner: Spawner) -> ! {

    let p = embassy_stm32::init(Default::default());
    let mut i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz(200_000), Default::default());

    let led = LedDriver::new(&mut i2c).expect("failed to init led");
    match LedDriver::new(&mut i2d) {
        Ok(led) => {},
        Err(e) => {
            defmt::info!("Failed to initialize {e}");
        }
    }

    // Set brightness for channel 0 to maximum
    if let Err(e) = led.set_channel_brightness(LedChannel::Channel0, 255) {
        defmt::info!("Failed to set brightness: {e}");
    } else {
        defmt::info!("Successfully set brightness for Channel 0");
    }

    // Optionally, set brightness for multiple channels
    let channels = vec![(LedChannel::Channel1, 128), (LedChannel::Channel2, 64)];
    if let Err(e) = led.set_multiple_brightness(&channels) {
        defmt::info!("Failed to set multiple brightness: {e}");
    } else {
        defmt::info!("Successfully set brightness for multiple channels");
    }

    loop {} // Keep the program running
}