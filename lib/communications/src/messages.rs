//use core::time;

//use defmt::timestamp;
use hyped_can::HypedCanFrame;
//, Timestamp};
//use hyped_state_machine::states::State;

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
    PrechargeComplete { board: Board, voltage: Voltage },
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
            CanMessage::StartPrechargeCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StartPrechargeCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StartDischargeCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StartDischargeCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PrechargeStarted(board) => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U32, MessageIdentifier::PrechargeStarted);
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeStarted(board) => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U32, MessageIdentifier::DischargeStarted);
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PrechargeFailed { board, reason } => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U8, MessageIdentifier::PrechargeFailed);
                let data = CanData::U8(reason as u8).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::PrechargeComplete { board, voltage } => {
                let can_id = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::PrechargeComplete,
                );
                let data = CanData::U32(voltage.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::DischargeComplete { board, voltage } => {
                let can_id = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::DischargeComplete,
                );
                let data = CanData::U32(voltage.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }

            // Levitation
            CanMessage::StartLevitationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StartLevitationCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StopLevitationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StopLevitationCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStarted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::LevitationStarted,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStopped(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::LevitationStopped,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationFailed { board, reason } => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U8, MessageIdentifier::LevitationFailed);
                let data = CanData::U8(reason as u8).into();
                HypedCanFrame::new(can_id.into(), data)
            }

            // Dynamics
            CanMessage::ClampBrakesCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::ClampBrakesCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::UnclampBrakesCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::UnclampBrakesCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::ExtendLateralSuspensionCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::ExtendLateralSuspensionCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::RetractLateralSuspensionCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::RetractLateralSuspensionCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            CanMessage::BrakesClamped(board) => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U32, MessageIdentifier::BrakesClamped);
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::BrakesUnclamped(board) => {
                let can_id: CanId =
                    CanId::new(board, CanDataType::U32, MessageIdentifier::BrakesUnclamped);
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionExtended(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::LateralSuspensionExtended,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionRetracted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::LateralSuspensionRetracted,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            // Propulsion
            CanMessage::StartPropulsionAccelerationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StartPropulsionAccelerationCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StartPropulsionBrakingCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::StartPropulsionBrakingCommand,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionAccelerationStarted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::PropulsionAccelerationStarted,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionBrakingStarted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::PropulsionBrakingStarted,
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
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

            // Calibration
            MessageIdentifier::StartCalibrationCommand => CanMessage::StartCalibrationCommand,
            MessageIdentifier::CalibrationComplete => CanMessage::CalibrationComplete(board),

            // Electronics
            MessageIdentifier::StartPrechargeCommand => CanMessage::StartPrechargeCommand,
            MessageIdentifier::StartDischargeCommand => CanMessage::StartDischargeCommand,
            MessageIdentifier::PrechargeStarted => CanMessage::PrechargeStarted(board),
            MessageIdentifier::DischargeStarted => CanMessage::DischargeStarted(board),
            MessageIdentifier::PrechargeComplete => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U32(voltage) => CanMessage::PrechargeComplete {
                        board,
                        voltage: Voltage(voltage),
                    },
                    _ => panic!("Invalid CanData for PrechargeComplete"),
                }
            }
            MessageIdentifier::DischargeComplete => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U32(voltage) => CanMessage::DischargeComplete {
                        board,
                        voltage: Voltage(voltage),
                    },
                    _ => panic!("Invalid CanData for DischargeComplete"),
                }
            }
            MessageIdentifier::PrechargeFailed => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U8(reason_u8) => CanMessage::PrechargeFailed {
                        board,
                        reason: Reason::try_from(reason_u8).unwrap(),
                    },
                    _ => panic!("Invalid CanData for PrechargeFailed"),
                }
            }

            // Levitation
            MessageIdentifier::StartLevitationCommand => CanMessage::StartLevitationCommand,
            MessageIdentifier::StopLevitationCommand => CanMessage::StopLevitationCommand,
            MessageIdentifier::LevitationStarted => CanMessage::LevitationStarted(board),
            MessageIdentifier::LevitationStopped => CanMessage::LevitationStopped(board),
            MessageIdentifier::LevitationFailed => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U8(reason_u8) => CanMessage::LevitationFailed {
                        board,
                        reason: Reason::try_from(reason_u8).unwrap(),
                    },
                    _ => panic!("Invalid CanData for LevitationFailed"),
                }
            }

            // Dynamics
            MessageIdentifier::UnclampBrakesCommand => CanMessage::UnclampBrakesCommand,
            MessageIdentifier::ClampBrakesCommand => CanMessage::ClampBrakesCommand,
            MessageIdentifier::RetractLateralSuspensionCommand => {
                CanMessage::RetractLateralSuspensionCommand
            }
            MessageIdentifier::ExtendLateralSuspensionCommand => {
                CanMessage::ExtendLateralSuspensionCommand
            }
            MessageIdentifier::BrakesClamped => CanMessage::BrakesClamped(board),
            MessageIdentifier::BrakesUnclamped => CanMessage::BrakesUnclamped(board),
            MessageIdentifier::LateralSuspensionRetracted => {
                CanMessage::LateralSuspensionRetracted(board)
            }
            MessageIdentifier::LateralSuspensionExtended => {
                CanMessage::LateralSuspensionExtended(board)
            }

            // Propulsion
            MessageIdentifier::StartPropulsionAccelerationCommand => {
                CanMessage::StartPropulsionAccelerationCommand
            }
            MessageIdentifier::StartPropulsionBrakingCommand => {
                CanMessage::StartPropulsionBrakingCommand
            }
            MessageIdentifier::PropulsionAccelerationStarted => {
                CanMessage::PropulsionAccelerationStarted(board)
            }
            MessageIdentifier::PropulsionBrakingStarted => {
                CanMessage::PropulsionBrakingStarted(board)
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
        boards::Board, data::CanData, heartbeat::Heartbeat, measurements::MeasurementReading,
        messages::CanMessage,
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
