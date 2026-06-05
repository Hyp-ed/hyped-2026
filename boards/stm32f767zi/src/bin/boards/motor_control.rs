#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler},
    peripherals::{CAN1, CAN3},
};
use embassy_time::{Duration, Timer};
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    default_can_config,
    tasks::{
        can::{
            receive::can_receiver,
            send::can_sender,
        },
        motor_control::{
            control::motor_control_loop, 
            receive::motor_rx_task, 
            control::motor_setup_task},
    }
};
use hyped_communications::boards::Board;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
    CAN3_RX0 => Rx0InterruptHandler<CAN3>;
    CAN3_RX1 => Rx1InterruptHandler<CAN3>;
    CAN3_SCE => SceInterruptHandler<CAN3>;
    CAN3_TX => TxInterruptHandler<CAN3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    THIS_BOARD
        .init(Board::MotorControl)
        .expect("Failed to initialize board");

    let p = embassy_stm32::init(Default::default());

    info!("Setting up CAN1 (main bus)...");
    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can);
    can.enable().await;
    let (can_tx, can_rx) = can.split();
    spawner.must_spawn(can_receiver(can_rx));
    spawner.must_spawn(can_sender(can_tx));
    info!("CAN1 enabled");

    info!("Setting up CAN3 (motor bus)...");
    let mut can3 = Can::new(p.CAN3, p.PB3, p.PB4, Irqs);
    default_can_config!(can3);
    can3.enable().await;
    let (can3_tx, can3_rx) = can3.split();
    info!("CAN3 enabled");


    spawner.must_spawn(motor_rx_task(can3_rx));
    spawner.must_spawn(motor_setup_task());
    spawner.must_spawn(motor_control_loop(can3_tx));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
