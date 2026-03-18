#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    eth, gpio,
    peripherals::{self, CAN1},
    rng, Config,
};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    tasks::can::{receive::can_receiver, send::can_sender},
};
use hyped_communications::{boards::Board, bus::EVENT_BUS, events::Event};

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
        .init(Board::Sensors)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    let led: gpio::Output<'static> = gpio::Output::new(p.PB7, gpio::Level::High, gpio::Speed::Low);

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(sensors_board_response_task(led));

    loop {
        yield_now().await;
    }
}

#[embassy_executor::task]
async fn sensors_board_response_task(mut led: gpio::Output<'static>) -> ! {
    let rx = EVENT_BUS.receiver();

    loop {
        let event = rx.receive().await;

        match event {
            Event::StartPrechargeCommand => {
                EVENT_BUS.sender().send(Event::PrechargeStarted).await;

                led.set_high();

                embassy_time::Timer::after_secs(10).await;

                led.set_low();

                EVENT_BUS.sender().send(Event::PrechargeComplete).await;
            }
            _ => {} // Ignore other events
        }
    }
}
