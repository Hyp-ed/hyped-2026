#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{i2c::I2c, time::Hertz};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz(100_000), Default::default());

    for addr in 0x08u8..=0x77 {
        defmt::info!("Scanning addr {}", addr);
        match i2c.blocking_write(addr, &[]) {
            Ok(_) => defmt::info!("Found device at 0x{:02X}", addr),
            Err(_) => {}
        }
    }
    defmt::info!("Scan complete");
}
