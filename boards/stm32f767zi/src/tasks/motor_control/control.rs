use defmt::*;
use embassy_stm32::can::CanTx;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use hyped_can::HypedCanFrame;
use hyped_motors::can_open_message::CanOpenMessage;
use hyped_motors::can_open_processor::Messages;

const NODE_ID: u8 = 0x01;

pub enum MotorCommand {
    SendMessage(Messages),
}

pub static MOTOR_COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, MotorCommand, 32> = Channel::new();

#[embassy_executor::task]
pub async fn motor_setup_task() {
    let sender = MOTOR_COMMAND_CHANNEL.sender();

    info!("Starting motor setup sequence...");

    let config_sequence = [
        Messages::TestModeCommand(0),
        Messages::ModesOfOperation,
        Messages::SensorType,
        Messages::UndervoltageLimit,
        Messages::TestStepperFrequency,
        Messages::TestStepperEnable,
        Messages::SetMaxCurrent,
        Messages::SecondaryCurrentProtection,
        Messages::MotorRatedCurrent,
        Messages::OvervoltageLimit,
    ];

    for msg in config_sequence {
        sender.send(MotorCommand::SendMessage(msg)).await;
        Timer::after(Duration::from_secs(1)).await;
    }

    info!("Config complete. Transitioning to Operational...");

    sender.send(MotorCommand::SendMessage(Messages::EnterPreoperationalState)).await;
    Timer::after(Duration::from_secs(30)).await;

    sender.send(MotorCommand::SendMessage(Messages::EnterOperationalState)).await;
    Timer::after(Duration::from_secs(1)).await;

    sender.send(MotorCommand::SendMessage(Messages::Shutdown)).await;
    Timer::after(Duration::from_secs(1)).await;

    sender.send(MotorCommand::SendMessage(Messages::SwitchOn)).await;
    Timer::after(Duration::from_secs(30)).await;

    sender.send(MotorCommand::SendMessage(Messages::StartDrive)).await;
    Timer::after(Duration::from_secs(15)).await;

    info!("Motor setup complete and drive started.");
}

#[embassy_executor::task]
pub async fn motor_control_loop(mut tx: CanTx<'static>) {
    let receiver = MOTOR_COMMAND_CHANNEL.receiver();

    loop {
        let command = receiver.receive().await;
        match command {
            MotorCommand::SendMessage(msg) => {
                send_motor_message(&mut tx, msg).await;
            }
        }
    }
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
