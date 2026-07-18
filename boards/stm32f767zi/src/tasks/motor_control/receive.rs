use defmt::*;
use embassy_stm32::can::CanRx;
use hyped_communications::{
    boards::Board, bus, emergency::Reason, events::Event, messages::CanMessage,
};

use crate::tasks::can::send::CAN_SEND;

const MOTOR_EMERGENCY_ID: u32 = 0x081;
const MOTOR_SDO_RESPONSE_ID: u32 = 0x581;

#[embassy_executor::task]
pub async fn motor_rx_task(mut rx: CanRx<'static>) {
    loop {
        match rx.read().await {
            Ok(envelope) => {
                let frame = envelope.frame;
                let raw_id: u32 = match frame.id() {
                    embassy_stm32::can::Id::Standard(id) => id.as_raw() as u32,
                    embassy_stm32::can::Id::Extended(id) => id.as_raw(),
                };
                info!(
                    "Motor RX: id={=u32:x} data={=[u8]:#02x}",
                    raw_id,
                    frame.data()
                );

                if motor_frame_is_error(raw_id, frame.data()) {
                    trigger_motor_emergency("motor controller reported an error").await;
                    return;
                }
            }
            Err(e) => {
                warn!("Motor CAN receive error: {:?}", e);
                continue;
            }
        }
    }
}

fn motor_frame_is_error(raw_id: u32, data: &[u8]) -> bool {
    let emergency_has_error_code = raw_id == MOTOR_EMERGENCY_ID
        && data.len() >= 2
        && u16::from_le_bytes([data[0], data[1]]) != 0;
    let sdo_was_aborted = raw_id == MOTOR_SDO_RESPONSE_ID && data.first() == Some(&0x80);

    emergency_has_error_code || sdo_was_aborted
}

async fn trigger_motor_emergency(message: &str) {
    error!("{}", message);
    CAN_SEND
        .send(CanMessage::Emergency(Board::MotorControl, Reason::Unknown))
        .await;
    bus::publish(Event::Emergency {
        from: Board::MotorControl,
        reason: Reason::Unknown,
    })
    .await;
}
