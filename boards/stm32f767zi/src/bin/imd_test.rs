#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    peripherals::CAN1,
};
use hyped_boards_stm32f767zi::{
    default_can_config,
    tasks::{can::receive::can_receiver, sensors::read_imd},
};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (_, can_rx) = can.split();

    // while let Ok(envelope) = can_rx.read().await {
    // defmt::info!("{}", envelope);
    // }

    // defmt::info!("{}", can_rx.read().await);
    // defmt::info!("{}", can_rx.read().await);
    spawner.must_spawn(can_receiver(can_rx));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(read_imd::read_imd());

    loop {
        yield_now().await;
    }
}
