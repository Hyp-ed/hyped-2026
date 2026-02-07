#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hyped_boards_stm32f767zi::sdmmc::HypedSdmmc;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let hyped_sdmmc = HypedSdmmc::new().await;

    hyped_sdmmc.read_logs().await.unwrap();

    loop {
        core::hint::spin_loop();
    }
}
