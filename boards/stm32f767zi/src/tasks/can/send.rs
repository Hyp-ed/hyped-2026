use embassy_stm32::can::{CanTx, ExtendedId, Frame, Id};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{with_timeout, Duration};
use hyped_can::HypedCanFrame;
use hyped_communications::messages::CanMessage;

/// Channel for sending CAN messages.
pub static CAN_SEND: Channel<CriticalSectionRawMutex, CanMessage, 32> = Channel::new();

/// Task that sends CAN messages from a channel.
#[embassy_executor::task]
pub async fn can_sender(mut tx: CanTx<'static>) {
    let can_sender = CAN_SEND.receiver();

    // Clear the tx buffer
    tx.flush_all().await;
    defmt::info!("Starting...");

    loop {
        let message = can_sender.receive().await;

        let log_command = matches!(
            &message,
            CanMessage::MotorControllerSetupCommand
                | CanMessage::MotorControllerSetOperationalCommand
                | CanMessage::MotorControllerSetupComplete
                | CanMessage::MotorControllerOperational
                | CanMessage::StartPropulsionAccelerationCommand
                | CanMessage::StartPropulsionBrakingCommand
                | CanMessage::PropulsionAccelerationStarted
                | CanMessage::PropulsionBrakingStarted
                | CanMessage::ClampBrakesCommand
                | CanMessage::UnclampBrakesCommand
        );

        if log_command {
            defmt::info!("CAN TX dequeued: {:?}", message);
        }

        defmt::debug!("Sending CAN message: {:?}", message);

        let can_frame: HypedCanFrame = message.into();

        let id = Id::Extended(ExtendedId::new(can_frame.can_id).unwrap());
        let data = can_frame.data;

        let frame = Frame::new_data(id, &data).unwrap();

        if with_timeout(Duration::from_millis(100), tx.write(&frame))
            .await
            .is_err()
        {
            defmt::warn!("CAN TX blocked for >100ms while sending {:?}", frame);
            tx.write(&frame).await;
        }

        if log_command {
            defmt::info!("CAN TX complete: {:?}", frame);
        }
        defmt::debug!("CAN message sent: {:?}", frame);
    }
}
