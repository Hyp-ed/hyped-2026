use core::time;

use defmt::timestamp;
use hyped_can::{HypedCanFrame, Timestamp};
use hyped_state_machine::states::State;

use crate::{boards::Board, emergency::Reason, events::Voltage};

use super::{
    can_id::CanId,
    data::{CanData, CanDataType},
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    message_identifier::MessageIdentifier,
};

#[derive(PartialEq, Debug, Clone, defmt::Format)]
pub enum CanMessage {
    // --- Note ----
    // Use measurementReading for all sensor monitoring
    // For important data which FSM needs to know to make decisions, include in payload

    // Existing
    MeasurementReading(MeasurementReading),
    Heartbeat(Heartbeat),
    Emergency(Board, Reason),

    // Calibration
    StartCalibrationCommand,
    CalibrationComplete(Board),

    // Electronics
    StartPrechargeCommand,
    StartDischargeCommand,
    PrechargeStarted(Board),
    DischargeStarted(Board),

    // Includes Data
    PrechargeCompleteVoltage { board: Board, voltage: Voltage },
    DischargeComplete { board: Board, voltage: Voltage },
    PrechargeFailed { board: Board, reason: Reason },

    // Levitation
    StartLevitationCommand,
    StopLevitationCommand,

    // Includes Data
    LevitationStarted(Board),
    LevitationStopped(Board),
    LevitationFailed { board: Board, reason: Reason },

    // Dynamics
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,

    BrakesClamped(Board),
    BrakesUnclamped(Board),
    LateralSuspensionRetracted(Board),
    LateralSuspensionExtended(Board),

    // Propulsion
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,
    PropulsionAccelerationStarted(Board),
    PropulsionBrakingStarted(Board),
}

// Converts a CanMessage into a HypedCanFrame ready to be sent over the CAN bus
impl From<CanMessage> for HypedCanFrame {
    fn from(val: CanMessage) -> Self {
        match val {
            // Existing
            CanMessage::MeasurementReading(measurement_reading) => {
                let message_identifier =
                    MessageIdentifier::Measurement(measurement_reading.measurement_id);
                let can_id = CanId::new(
                    measurement_reading.board,
                    measurement_reading.reading.into(),
                    message_identifier,
                );
                HypedCanFrame::new(can_id.into(), measurement_reading.reading.into())
            }
            CanMessage::Heartbeat(heartbeat) => {
                let can_id = CanId::new(
                    heartbeat.from,
                    CanDataType::Heartbeat,
                    MessageIdentifier::Heartbeat,
                );
                HypedCanFrame::new(can_id.into(), CanData::Heartbeat(heartbeat.to).into())
            }
            CanMessage::Emergency(board, reason) => {
                let can_id =
                    CanId::new(board, CanDataType::Emergency, MessageIdentifier::Emergency);
                HypedCanFrame::new(can_id.into(), CanData::Emergency(reason).into())
            }

            // New

            // Calibration
            CanMessage::StartCalibrationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry, // TODO: find out if this is right
                    CanDataType::U32,
                    MessageIdentifier::StartCalibrationCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::CalibrationComplete(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::CalibrationComplete,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            // Electronics
            CanMessage::StartPrechargeCommand => {}
            CanMessage::StartDischargeCommand => {}
            CanMessage::PrechargeStarted(board) => {}
            CanMessage::DischargeStarted(board) => {}
            CanMessage::PrechargeFailed { board, reason } => {} // Levitation
        }
    }
}

// Converts an incoming HypedCanFrame read from the CAN bus into a CanMessage
impl From<HypedCanFrame> for CanMessage {
    fn from(frame: HypedCanFrame) -> Self {
        let can_id: CanId = frame.can_id.into();
        let message_identifier = can_id.message_identifier;
        let board = can_id.board;

        match message_identifier {
            // Existing
            MessageIdentifier::Measurement(measurement_id) => {
                let reading: CanData = frame.data.into();
                let measurement_reading = MeasurementReading {
                    reading,
                    board,
                    measurement_id,
                };
                CanMessage::MeasurementReading(measurement_reading)
            }
            MessageIdentifier::Heartbeat => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Heartbeat(to) => {
                        let heartbeat = Heartbeat::new(to, board);
                        CanMessage::Heartbeat(heartbeat)
                    }
                    _ => panic!("Invalid CanData for Heartbeat"),
                }
            }
            MessageIdentifier::Emergency => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Emergency(reason) => CanMessage::Emergency(board, reason),
                    _ => panic!("Invalid CanData for Emergency"),
                }
            }

            // New
            MessageIdentifier::CalibrationComplete => {
                let reading: CanData = frame.data.into();
                l
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hyped_can::HypedCanFrame;
    use hyped_core::config::MeasurementId;
    //use hyped_state_machine::states::State;

    use crate::{
        boards::Board,
        data::CanData,
        heartbeat::Heartbeat,
        measurements::MeasurementReading,
        messages::CanMessage,
        //state_transition::{StateTransitionCommand, StateTransitionRequest},
    };

    #[test]
    fn it_works() {
        let measurement_reading = MeasurementReading::new(
            CanData::F32(0.0),
            Board::Telemetry,
            MeasurementId::Acceleration,
        );
        let can_message = CanMessage::MeasurementReading(measurement_reading);

        let can_frame: HypedCanFrame = can_message.clone().into();
        let can_message_from_frame: CanMessage = can_frame.into();

        assert_eq!(can_message, can_message_from_frame)
    }

    #[test]
    fn it_works_heartbeat() {
        let heartbeat = CanMessage::Heartbeat(Heartbeat::new(Board::KeyenceTester, Board::Test));
        let can_frame: HypedCanFrame = heartbeat.clone().into();
        let can_message_from_frame: CanMessage = can_frame.into();
        assert_eq!(heartbeat, can_message_from_frame)
    }
}
