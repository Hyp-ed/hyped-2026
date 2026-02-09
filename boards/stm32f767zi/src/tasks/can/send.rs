use embassy_futures::select::{select, Either};
use embassy_stm32::can::{frame::Header, CanTx, ExtendedId, Frame, Id, StandardId};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    channel::{Channel, Sender},
};
use hyped_can::HypedCanFrame;
use hyped_communications::messages::CanMessage;
use hyped_sensors::lp_bms::BMS_REQUEST_ID;

use crate::{
    sdmmc::logging::{LogBufWriter, MESSAGE_SIZE_RAW},
    send_log,
};

pub fn bms_frame(cmd: [u8; 8]) -> Option<Frame> {
    Frame::new(
        Header::new(
            embassy_stm32::can::Id::Standard(StandardId::new(BMS_REQUEST_ID as u16)?),
            0,
            false,
        ),
        &cmd,
    )
    .ok()
}

/// Channel for sending CAN messages.
pub static CAN_SEND: Channel<CriticalSectionRawMutex, CanMessage, 10> = Channel::new();
/// Channel for BMS messages since they are different than other CAN messages
pub static BMS_SEND: Channel<CriticalSectionRawMutex, [u8; 8], 4> = Channel::new();

/// Task that sends CAN messages from a channel.
#[embassy_executor::task]
pub async fn can_sender(
    mut tx: CanTx<'static>,
    log_sender: Option<Sender<'static, ThreadModeRawMutex, [u8; MESSAGE_SIZE_RAW], 4>>,
) {
    let can_sender = CAN_SEND.receiver();
    let bms_sender = BMS_SEND.receiver();

    // Clear the tx buffer
    tx.flush_all().await;
    defmt::info!("Starting...");

    loop {
        let message = select(can_sender.receive(), bms_sender.receive()).await;

        match message {
            Either::First(message) => {
                defmt::debug!("Sending CAN message: {:?}", message);

                // Log it to the SD Card
                send_log!(log_sender, "Sent: {:#?}", message);
                let can_frame: HypedCanFrame = message.into();

                let id = Id::Extended(ExtendedId::new(can_frame.can_id).unwrap());
                let data = can_frame.data;

                let frame = Frame::new_data(id, &data).unwrap();

                tx.write(&frame).await;
                defmt::debug!("CAN message sent: {:?}", frame);
            }

            Either::Second(cmd) => {
                if let Some(frame) = bms_frame(cmd) {
                    tx.write(&frame).await;
                    defmt::debug!("CAN message sent: {:?}", frame);
                }
            }
        }
    }
}
