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
    rng, time, Config,
};
use embassy_time::Timer;
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
        .init(Board::Sensors2)
        .expect("Failed to initialize board");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    let gpio1: gpio::Output<'static> =
        gpio::Output::new(p.PF13, gpio::Level::High, gpio::Speed::Low);
    let gpio2: gpio::Output<'static> =
        gpio::Output::new(p.PE9, gpio::Level::High, gpio::Speed::Low);
    let gpio3: gpio::Output<'static> =
        gpio::Output::new(p.PE11, gpio::Level::High, gpio::Speed::Low);
    let gpio4: gpio::Output<'static> =
        gpio::Output::new(p.PF14, gpio::Level::High, gpio::Speed::Low);
    let gpio5: gpio::Output<'static> =
        gpio::Output::new(p.PE13, gpio::Level::High, gpio::Speed::Low);
    let gpio6: gpio::Output<'static> =
        gpio::Output::new(p.PF15, gpio::Level::High, gpio::Speed::Low);

    let gpio_pins = GpioPins {
        gpio1,
        gpio2,
        gpio3,
        gpio4,
        gpio5,
        gpio6,
    };

    defmt::info!("Setting up CAN...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    defmt::info!("CAN setup complete");

    spawner.must_spawn(sensors_board_response_task(gpio_pins));

    loop {
        yield_now().await;
    }
}

struct GpioPins {
    gpio1: gpio::Output<'static>,
    gpio2: gpio::Output<'static>,
    gpio3: gpio::Output<'static>,
    gpio4: gpio::Output<'static>,
    gpio5: gpio::Output<'static>,
    gpio6: gpio::Output<'static>,
}

#[embassy_executor::task]
async fn sensors_board_response_task(mut gpio_pins: GpioPins) {
    let rx = EVENT_BUS.receiver();

    loop {
        let event = rx.receive().await;

        match event {
            Event::StartPrechargeCommand => {
                EVENT_BUS.sender().send(Event::PrechargeStarted).await;

                gpio_pins.gpio1.set_high();
                gpio_pins.gpio4.set_high();

                gpio_pins.gpio2.set_low();
                EVENT_BUS
                    .sender()
                    .send(Event::ShutdownCircuitryRelayClosed)
                    .await;

                gpio_pins.gpio3.set_low();
                EVENT_BUS
                    .sender()
                    .send(Event::ShutdownCircuitryRelayClosed)
                    .await;

                Timer::after_secs(4).await;
                gpio_pins.gpio4.set_low();

                gpio_pins.gpio2.set_high();
                gpio_pins.gpio3.set_high();

                EVENT_BUS
                    .sender()
                    .send(Event::BatteryPrechargeRelayOpen)
                    .await;

                EVENT_BUS
                    .sender()
                    .send(Event::MotorControllerRelayOpen)
                    .await;

                // there is a possibility that after 20s of this relay being on it needs to be turned back off, please have code for this commented for now until further confirmation
                // Timer::after_secs(20).await;
                //
                // gpio_pins.gpio2.set_low();
                // gpio_pins.gpio3.set_low();

                EVENT_BUS.sender().send(Event::PrechargeComplete).await;
            }

            Event::Emergency { from, reason } => {
                defmt::warn!("EMERGENCY: from {:?} reason={}", from, reason);
                return;
            }

            Event::Heartbeat { from } => {
                defmt::debug!("Heartbeat from {:?}", from);
            }

            _ => {} // Ignore other events
        }
    }
}
