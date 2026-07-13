#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;

fn brake_gpio(p: peripherals::PF15) -> Output<'static> {
    Output::new(p, Level::High, Speed::Low)
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let _brake_gpio = brake_gpio(p.PF15);

    defmt::info!("Brake GPIO set high");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
