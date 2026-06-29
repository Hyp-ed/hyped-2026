use defmt::*;
use embassy_stm32::can::CanTx;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use hyped_can::HypedCanFrame;
use hyped_communications::{bus::DynSubscriber, events::Event, messages::CanMessage};
use hyped_motors::{can_open_message::CanOpenMessage, can_open_processor::Messages};

use crate::tasks::can::send::CAN_SEND;

const NODE_ID: u8 = 0x01;

pub enum MotorCommand {
    SendMessage(Messages),
}

pub static MOTOR_COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, MotorCommand, 32> =
    Channel::new();

#[embassy_executor::task]
pub async fn motor_command_task(mut events: DynSubscriber<'static, Event>) {
    let mut setup_complete = false;
    let mut operational = false;

    loop {
        match events.next_message_pure().await {
            Event::MotorControllerSetupCommand => {
                if setup_complete {
                    info!("Motor controller setup already complete");
                    CAN_SEND.send(CanMessage::MotorControllerSetupComplete).await;
                    continue;
                }

                run_setup_sequence().await;
                setup_complete = true;
                CAN_SEND.send(CanMessage::MotorControllerSetupComplete).await;
            }
            Event::MotorControllerSetOperationalCommand => {
                if !setup_complete {
                    warn!("Ignoring operational command before motor controller setup is complete");
                    continue;
                }
                if operational {
                    info!("Motor controller already operational");
                    CAN_SEND.send(CanMessage::MotorControllerOperational).await;
                    continue;
                }

                run_operational_sequence().await;
                operational = true;
                CAN_SEND.send(CanMessage::MotorControllerOperational).await;
            }
            Event::StartPropulsionAccelerationCommand => {
                if !operational {
                    warn!("Ignoring acceleration command before motor controller is operational");
                    continue;
                }

                info!("Starting low-power test acceleration");
                send_motor_command(Messages::TestModeCommand(200)).await;
                Timer::after(Duration::from_secs(3)).await;
                CAN_SEND.send(CanMessage::PropulsionAccelerationStarted).await;
            }
            _ => {}
        }
    }
}

async fn run_setup_sequence() {
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
        send_motor_command(msg).await;
        Timer::after(Duration::from_secs(1)).await;
    }

    info!("Config complete. Transitioning to Operational...");
}

async fn run_operational_sequence() {
    send_motor_command(Messages::EnterPreoperationalState).await;
    Timer::after(Duration::from_secs(30)).await;

    send_motor_command(Messages::EnterOperationalState).await;
    Timer::after(Duration::from_secs(1)).await;

    send_motor_command(Messages::Shutdown).await;
    Timer::after(Duration::from_secs(1)).await;

    send_motor_command(Messages::SwitchOn).await;
    Timer::after(Duration::from_secs(30)).await;

    send_motor_command(Messages::StartDrive).await;
    Timer::after(Duration::from_secs(15)).await;

    info!("Motor controller operational and drive started.");
}

async fn send_motor_command(message: Messages) {
    MOTOR_COMMAND_CHANNEL
        .send(MotorCommand::SendMessage(message))
        .await;
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
    let id = embassy_stm32::can::Id::Standard(unwrap!(embassy_stm32::can::StandardId::new(0x000)));

    let data = [command, node_id];
    let frame = embassy_stm32::can::Frame::new_data(id, &data).unwrap();
    tx.write(&frame).await;
}

async fn send_sdo(tx: &mut CanTx<'static>, msg: CanOpenMessage) {
    let hyped_frame: HypedCanFrame = msg.into();

    let id = if hyped_frame.can_id <= 0x7FF {
        embassy_stm32::can::Id::Standard(unwrap!(embassy_stm32::can::StandardId::new(
            hyped_frame.can_id as u16
        )))
    } else {
        embassy_stm32::can::Id::Extended(unwrap!(embassy_stm32::can::ExtendedId::new(
            hyped_frame.can_id
        )))
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
