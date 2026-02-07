use embassy_stm32::can::{CanRx, Id};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    channel::{Channel, Sender},
};
use hyped_can::HypedCanFrame;
use hyped_communications::{
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    messages::CanMessage,
    state_transition::{StateTransitionCommand, StateTransitionRequest},
};

use crate::{
    board_state::EMERGENCY,
    sdmmc::logging::{LogBufWriter, MESSAGE_SIZE_RAW},
    send_log,
};

use defmt_rtt as _;
use panic_probe as _;

/// Stores incoming state transitions received from CAN.
/// All boards should listen to this channel and update their states accordingly.
pub static INCOMING_STATE_TRANSITION_COMMANDS: Channel<
    CriticalSectionRawMutex,
    StateTransitionCommand,
    10,
> = Channel::new();

/// Stores incoming state transition requests received from CAN.
/// Only used by the main control board running the state_machine task.
pub static INCOMING_STATE_TRANSITION_REQUESTS: Channel<
    CriticalSectionRawMutex,
    StateTransitionRequest,
    10,
> = Channel::new();

/// Stores heartbeat messages coming in from other boards that we need to respond to.
pub static INCOMING_HEARTBEATS: Channel<CriticalSectionRawMutex, Heartbeat, 10> = Channel::new();

/// Stores measurement readings coming in from other boards.
pub static INCOMING_MEASUREMENTS: Channel<CriticalSectionRawMutex, MeasurementReading, 10> =
    Channel::new();

/// Task that receives CAN messages and puts them into a channel.
/// Currently only supports `StateTransitionCommand`, `StateTransitionRequest` and `Heartbeat` messages.
#[embassy_executor::task]
pub async fn can_receiver(
    mut rx: CanRx<'static>,
    log_sender: Option<Sender<'static, ThreadModeRawMutex, [u8; MESSAGE_SIZE_RAW], 4>>,
) {
    let emergency_sender = EMERGENCY.sender();
    let state_transition_commands_sender = INCOMING_STATE_TRANSITION_COMMANDS.sender();
    let state_transition_requests_sender = INCOMING_STATE_TRANSITION_REQUESTS.sender();
    let incoming_heartbeat_sender = INCOMING_HEARTBEATS.sender();

    loop {
        defmt::debug!("Waiting for CAN message");

        let envelope = rx.read().await;
        if envelope.is_err() {
            continue;
        }
        let envelope = envelope.unwrap();

        let id = envelope.frame.id();
        let can_id = match id {
            Id::Standard(id) => {
                let raw_id = id.as_raw() as u32;

                // TODO: figure out if this can bus is in the main can bus
                // // is this a bms message
                // if raw_id == BMS_RESPONSE_ID {
                //     INCOMING_BMS_MESSAGES.sender().send(());
                //
                //     send_log!(log_sender, "Received BMS: {:#?}", envelope);
                //
                //     continue 'recv_loop;
                // }

                raw_id
            } // 11-bit ID
            Id::Extended(id) => id.as_raw(), // 29-bit ID
        };
        let mut data = [0u8; 8];
        data.copy_from_slice(envelope.frame.data());
        let can_frame = HypedCanFrame::new(can_id, data);

        let can_message: CanMessage = can_frame.into();
        defmt::debug!("Received CAN message: {:?}", can_message);

        // Log it to the SD Card
        send_log!(log_sender, "Received: {:#?}", can_message);

        match can_message {
            CanMessage::StateTransitionCommand(state_transition_command) => {
                state_transition_commands_sender
                    .send(state_transition_command)
                    .await;
            }
            // Requests will only be used on the primary board running the state_machine task.
            CanMessage::StateTransitionRequest(state_transition) => {
                state_transition_requests_sender
                    .send(state_transition)
                    .await;
            }
            CanMessage::Heartbeat(heartbeat) => {
                defmt::debug!("Received heartbeat: {:?}", heartbeat);
                incoming_heartbeat_sender.send(heartbeat).await;
            }
            CanMessage::Emergency(board, reason) => {
                emergency_sender.send(true);
                defmt::error!("Emergency message from board {}: {}", board, reason);
            }
            CanMessage::MeasurementReading(measurement_reading) => {
                defmt::info!("Received measurement reading: {:?}", measurement_reading);
                INCOMING_MEASUREMENTS.send(measurement_reading).await;
            }
        }
    }
}
