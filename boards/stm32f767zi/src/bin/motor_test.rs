#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, CanRx, CanTx, Fifo, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    peripherals::CAN1,
};
use embassy_time::{Duration, Timer};
use hyped_can::HypedCanFrame;
use hyped_motors::can_open_message::CanOpenMessage;
use hyped_motors::can_open_processor::Messages;
use panic_probe as _;
use static_cell::StaticCell;

const NODE_ID: u8 = 0x01;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Setting up CAN1 (main bus)...");
    static CAN: StaticCell<Can<'static>> = StaticCell::new();
    let can = CAN.init(Can::new(p.CAN1, p.PD0, p.PD1, Irqs));

    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    can.modify_config().set_bitrate(500_000);
    can.enable().await;

    info!("CAN1 enabled");

    let (mut tx, rx) = can.split();
    unwrap!(spawner.spawn(rx_task(rx)));

    info!("Resetting test mode...");
    send_motor_message(&mut tx, Messages::TestModeCommand(0)).await;
    wait_after_send().await;

    info!("Sending Modes of Operation message...");
    send_motor_message(&mut tx, Messages::ModesOfOperation).await;
    wait_after_send().await;

    info!("Sending Sensor Type message...");
    send_motor_message(&mut tx, Messages::SensorType).await;
    wait_after_send().await;

    info!("Sending Undervoltage Limit message...");
    send_motor_message(&mut tx, Messages::UndervoltageLimit).await;
    wait_after_send().await;

    info!("Sending Test Stepper frequency message...");
    send_motor_message(&mut tx, Messages::TestStepperFrequency).await;
    wait_after_send().await;

    info!("Sending Test Stepper Enable message...");
    send_motor_message(&mut tx, Messages::TestStepperEnable).await;
    wait_after_send().await;

    info!("Setting maximum controller current...");
    send_motor_message(&mut tx, Messages::SetMaxCurrent).await;
    wait_after_send().await;

    info!("Setting secondary current protection...");
    send_motor_message(&mut tx, Messages::SecondaryCurrentProtection).await;
    wait_after_send().await;

    info!("Setting motor rated current...");
    send_motor_message(&mut tx, Messages::MotorRatedCurrent).await;
    wait_after_send().await;

    info!("Setting overvoltage limit...");
    send_motor_message(&mut tx, Messages::OvervoltageLimit).await;
    wait_after_send().await;

    info!("CONFIG COMPLETE");
    Timer::after(Duration::from_secs(5)).await;

    info!("Sending Preoperational message...");
    send_motor_message(&mut tx, Messages::EnterPreoperationalState).await;
    Timer::after(Duration::from_secs(30)).await;
    wait_after_send().await;

    info!("Sending Operational message...");
    send_motor_message(&mut tx, Messages::EnterOperationalState).await;
    wait_after_send().await;

    info!("Sending Shutdown message...");
    send_motor_message(&mut tx, Messages::Shutdown).await;
    wait_after_send().await;

    info!("Sending Switch On message...");
    send_motor_message(&mut tx, Messages::SwitchOn).await;
    wait_after_send().await;
    Timer::after(Duration::from_secs(30)).await;

    info!("Sending Start Drive message...");
    send_motor_message(&mut tx, Messages::StartDrive).await;
    Timer::after(Duration::from_secs(15)).await;

    info!("Starting motor");
    info!("Incrementing Test Mode Command to 1000...");

    info!("Setting Test Mode Command to 200");
    send_motor_message(&mut tx, Messages::TestModeCommand(200)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Setting Test Mode Command to 400");
    send_motor_message(&mut tx, Messages::TestModeCommand(400)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Setting Test Mode Command to 600");
    send_motor_message(&mut tx, Messages::TestModeCommand(600)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Setting Test Mode Command to 800");
    send_motor_message(&mut tx, Messages::TestModeCommand(800)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Setting Test Mode Command to 1000");
    send_motor_message(&mut tx, Messages::TestModeCommand(1000)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Ramping down Test Mode Command to 0...");
    send_motor_message(&mut tx, Messages::TestModeCommand(500)).await;
    Timer::after(Duration::from_secs(3)).await;

    info!("Stopping motor...");
    send_motor_message(&mut tx, Messages::EnterStopState).await;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn rx_task(mut rx: CanRx<'static>) {
    loop {
        match rx.read().await {
            Ok(envelope) => {
                let frame = envelope.frame;
                let raw_id: u32 = match frame.id() {
                    embassy_stm32::can::Id::Standard(id) => id.as_raw() as u32,
                    embassy_stm32::can::Id::Extended(id) => id.as_raw(),
                };
                info!("Received id={=u32:x} data={=[u8]:#02x}", raw_id, frame.data());
            }
            Err(e) => {
                warn!("CAN receive error: {:?}", e);
                continue;
            }
        }
    }
}

async fn wait_after_send() {
    Timer::after(Duration::from_secs(1)).await;
}

async fn send_nmt(tx: &mut CanTx<'static>, command: u8, node_id: u8) {
    let id = embassy_stm32::can::Id::Standard(unwrap!(
        embassy_stm32::can::StandardId::new(0x000)
    ));

    let data = [command, node_id];
    let frame = embassy_stm32::can::Frame::new_data(id, &data).unwrap();
    tx.write(&frame).await;
}

async fn send_sdo(tx: &mut CanTx<'static>, msg: CanOpenMessage) {
    let hyped_frame: HypedCanFrame = msg.into();

    let id = if hyped_frame.can_id <= 0x7FF {
        embassy_stm32::can::Id::Standard(unwrap!(
            embassy_stm32::can::StandardId::new(hyped_frame.can_id as u16)
        ))
    } else {
        embassy_stm32::can::Id::Extended(unwrap!(
            embassy_stm32::can::ExtendedId::new(hyped_frame.can_id)
        ))
    };

    let frame = embassy_stm32::can::Frame::new_data(id, &hyped_frame.data).unwrap();
    tx.write(&frame).await;
}

async fn send_motor_message(tx: &mut CanTx<'static>, msg: Messages) {
    match msg {
        Messages::EnterStopState => send_nmt(tx, 0x02, NODE_ID).await,
        Messages::EnterPreoperationalState => send_nmt(tx, 0x80, NODE_ID).await,
        Messages::EnterOperationalState => send_nmt(tx, 0x01, NODE_ID).await,
        other => {
            let can_open_msg: CanOpenMessage = other.into();
            send_sdo(tx, can_open_msg).await;
        }
    }
}