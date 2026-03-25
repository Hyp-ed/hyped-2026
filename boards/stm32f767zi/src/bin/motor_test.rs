#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    peripherals::CAN1,
};
use embassy_time::{Duration, Timer};
use hyped_motors::can_open_processor::Messages;
use hyped_motors::can_open_message::CanOpenMessage;
use hyped_can::HypedCanFrame;
use panic_probe as _;
use static_cell::StaticCell;

// bind_interrupts!(struct Irqs {
//     CAN2_RX0 => Rx0InterruptHandler<CAN2>;
//     CAN2_RX1 => Rx1InterruptHandler<CAN2>;
//     CAN2_SCE => SceInterruptHandler<CAN2>;
//     CAN2_TX => TxInterruptHandler<CAN2>;
// });

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Setting up CAN1 (main bus)...");
    static CAN: StaticCell<Can<'static>> = StaticCell::new();
    // Using pins from motor_control.rs: PB5, PB6
    let can = CAN.init(Can::new(p.CAN1, p.PD0, p.PD1, Irqs));
    
    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    can.modify_config().set_bitrate(500_000);
    can.enable().await;
    
    info!("CAN1 enabled");

    let (mut tx, _rx) = can.split();

    //Set up stuff
    info!("Sending Test Stepper frequency message...");
    send_motor_message(&mut tx, Messages::TestStepperFrequency).await;

    info!("Sending Test Stepper Enable message...");
    send_motor_message(&mut tx, Messages::TestStepperEnable).await;

    info!("Setting maximum controller current...");
    send_motor_message(&mut tx, Messages::SetMaxCurrent).await;

    info!("Setting secondary current protection...");
    send_motor_message(&mut tx, Messages::SecondaryCurrentProtection).await;

    info!("Setting motor rated current...");
    send_motor_message(&mut tx, Messages::MotorRatedCurrent).await;

    info!("Setting overvoltage limit...");
    send_motor_message(&mut tx, Messages::OvervoltageLimit).await;

    info!("CONFIG COMPLETE");
    Timer::after(Duration::from_secs(5)).await;



    info!("Sending Preoperational message...");
    send_motor_message(&mut tx, Messages::EnterPreoperationalState).await;
    Timer::after(Duration::from_secs(30)).await;

    info!("Sending Operational message...");
    send_motor_message(&mut tx, Messages::EnterOperationalState).await;

    info!("Sending Shutdown message...");
    send_motor_message(&mut tx, Messages::Shutdown).await;

    info!("Sending Switch On message...");
    send_motor_message(&mut tx, Messages::SwitchOn).await;
    Timer::after(Duration::from_secs(30)).await;

    info!("Sending Start Drive message...");
    send_motor_message(&mut tx, Messages::StartDrive).await;
    Timer::after(Duration::from_secs(15)).await;

    info!("Sending Start Drive message...");
    send_motor_message(&mut tx, Messages::StartDrive).await;
    Timer::after(Duration::from_secs(2)).await;

    info!("Starting motor");
    info!("Setting Frequency to 200...");
    send_motor_message(&mut tx, Messages::SetFrequency(200)).await;
    Timer::after(Duration::from_secs(15)).await;

    info!("Stopping motor...");
    send_motor_message(&mut tx, Messages::QuickStop).await;
}

async fn send_motor_message(tx: &mut embassy_stm32::can::CanTx<'static>, msg: Messages) {
    let can_open_msg: CanOpenMessage = msg.into();
    let hyped_frame: HypedCanFrame = can_open_msg.into();
    
    // Convert HypedCanFrame to embassy_stm32::can::Frame
    let id = embassy_stm32::can::Id::Extended(unwrap!(embassy_stm32::can::ExtendedId::new(hyped_frame.can_id)));
    let frame = embassy_stm32::can::Frame::new_data(id, &hyped_frame.data).unwrap();
    
    tx.write(&frame).await;
}
