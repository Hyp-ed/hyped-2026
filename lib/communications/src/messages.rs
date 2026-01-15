//use core::time;

//use defmt::timestamp;
use hyped_can::HypedCanFrame;
//, Timestamp};
//use hyped_state_machine::states::State;

use crate::{
    boards::Board,
    emergency::Reason,
    events::{Airgap, Current, Force, Pressure, Temperature, Velocity, Voltage},
};

use super::{
    can_id::CanId,
    data::{CanData, CanDataType},
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    message_identifier::{EventId, MessageIdentifier},
};

#[derive(PartialEq, Debug, Clone, defmt::Format)]
pub enum CanMessage {
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
    PrechargeComplete {
        board: Board,
        voltage: Voltage,
    },
    DischargeComplete {
        board: Board,
        voltage: Voltage,
    },
    PrechargeFailed {
        board: Board,
        reason: Reason,
    },

    // Levitation
    StartLevitationCommand,
    StopLevitationCommand,
    LevitationSystemsReady,
    LevitationStarted {
        board: Board,
    },
    LevitationStopped {
        board: Board,
    },
    LevitationStatus {
        board: Board,
        current_ma: Current,
        airgap_μm: Airgap,
    },
    LevitationFailed {
        board: Board,
        reason: Reason,
    },

    // Dynamics
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,

    // TODO: moved pressure to DynamicsStatus, so send that event when needed
    BrakesClamped {
        board: Board,
    },
    BrakesUnclamped {
        board: Board,
    },
    LateralSuspensionRetracted {
        board: Board,
    },
    LateralSuspensionExtended {
        board: Board,
    },
    DynamicsStatus {
        board: Board,
        actuator_pressure_bar: Pressure,
    },

    // Propulsion
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,
    PropulsionAccelerationStarted,
    PropulsionBrakingStarted,

    PropulsionStatus {
        current_ma: Current,
        velocity_kmh: Velocity,
        temperature_c: Temperature,
        voltage_cv: Voltage,
    },
    PropulsionForce {
        force_n: Force, // calculated thrust force
    },
    PropulsionFailed {
        board: Board,
        reason: Reason,
    },
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
                    MessageIdentifier::Event(EventId::StartCalibrationCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::CalibrationComplete(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::CalibrationComplete),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            // Electronics
            CanMessage::StartPrechargeCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartPrechargeCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StartDischargeCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartDischargeCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PrechargeStarted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PrechargeStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeStarted(board) => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PrechargeFailed { board, reason } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U8,
                    MessageIdentifier::Event(EventId::PrechargeFailed),
                );
                let data = CanData::U8(reason as u8).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::PrechargeComplete { board, voltage } => {
                let can_id = CanId::new(
                    board,
                    CanDataType::U16,
                    MessageIdentifier::Event(EventId::PrechargeComplete),
                );
                let data = CanData::U16(voltage.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::DischargeComplete { board, voltage } => {
                let can_id = CanId::new(
                    board,
                    CanDataType::U16,
                    MessageIdentifier::Event(EventId::DischargeComplete),
                );
                let data = CanData::U16(voltage.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }

            // Levitation
            CanMessage::StartLevitationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartLevitationCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StopLevitationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StopLevitationCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationSystemsReady => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationSystemsReady),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStarted { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStopped { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationStopped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStatus {
                board,
                current_ma,
                airgap_μm,
            } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::TwoU16,
                    MessageIdentifier::Event(EventId::LevitationStatus),
                );
                let data: [u8; 8] = CanData::TwoU16([current_ma.0, airgap_μm.0]).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::LevitationFailed { board, reason } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U8,
                    MessageIdentifier::Event(EventId::LevitationFailed),
                );
                let data = CanData::U8(reason as u8).into();
                HypedCanFrame::new(can_id.into(), data)
            }

            // Dynamics
            CanMessage::ClampBrakesCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::ClampBrakesCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::UnclampBrakesCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::UnclampBrakesCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::ExtendLateralSuspensionCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::ExtendLateralSuspensionCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::RetractLateralSuspensionCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::RetractLateralSuspensionCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            CanMessage::BrakesClamped { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BrakesClamped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::BrakesUnclamped { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BrakesUnclamped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionExtended { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LateralSuspensionExtended),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionRetracted { board } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LateralSuspensionRetracted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DynamicsStatus {
                board,
                actuator_pressure_bar,
            } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U16,
                    MessageIdentifier::Event(EventId::DynamicsStatus),
                );
                let data = CanData::U16(actuator_pressure_bar.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }

            // Propulsion
            CanMessage::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry,
                    CanDataType::PropulsionStatus,
                    MessageIdentifier::Event(EventId::PropulsionStatus),
                );
                let data: [u8; 8] = CanData::PropulsionStatus {
                    current_ma: current_ma.0,
                    velocity_kmh: velocity_kmh.0,
                    temperature_c: temperature_c.0,
                    voltage_cv: voltage_cv.0,
                }
                .into();
                HypedCanFrame::new(can_id.into(), data)
            }

            CanMessage::PropulsionForce { force_n } => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, // TODO placeholder
                    CanDataType::U16,
                    MessageIdentifier::Event(EventId::PropulsionForce),
                );
                let data = CanData::U16(force_n.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::StartPropulsionAccelerationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartPropulsionAccelerationCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::StartPropulsionBrakingCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartPropulsionBrakingCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionAccelerationStarted => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, //TODO placeholder board for now
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PropulsionAccelerationStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionBrakingStarted => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, //TODO placeholder board for now
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PropulsionBrakingStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionFailed { board, reason } => {
                let can_id: CanId = CanId::new(
                    board,
                    CanDataType::U8,
                    MessageIdentifier::Event(EventId::PropulsionFailed),
                );
                let data = CanData::U8(reason as u8).into();
                HypedCanFrame::new(can_id.into(), data)
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
            MessageIdentifier::Event(EventId::StartCalibrationCommand) => {
                CanMessage::StartCalibrationCommand
            }
            MessageIdentifier::Event(EventId::CalibrationComplete) => {
                CanMessage::CalibrationComplete(board)
            }

            // Electronics
            MessageIdentifier::Event(EventId::StartPrechargeCommand) => {
                CanMessage::StartPrechargeCommand
            }
            MessageIdentifier::Event(EventId::StartDischargeCommand) => {
                CanMessage::StartDischargeCommand
            }
            MessageIdentifier::Event(EventId::PrechargeStarted) => {
                CanMessage::PrechargeStarted(board)
            }
            MessageIdentifier::Event(EventId::DischargeStarted) => {
                CanMessage::DischargeStarted(board)
            }
            MessageIdentifier::Event(EventId::PrechargeComplete) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(voltage) => CanMessage::PrechargeComplete {
                        board,
                        voltage: Voltage(voltage),
                    },
                    _ => panic!("Invalid CanData for PrechargeComplete"),
                }
            }
            MessageIdentifier::Event(EventId::DischargeComplete) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(voltage) => CanMessage::DischargeComplete {
                        board,
                        voltage: Voltage(voltage),
                    },
                    _ => panic!("Invalid CanData for DischargeComplete"),
                }
            }
            MessageIdentifier::Event(EventId::PrechargeFailed) => {
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
            MessageIdentifier::Event(EventId::StartLevitationCommand) => {
                CanMessage::StartLevitationCommand
            }
            MessageIdentifier::Event(EventId::StopLevitationCommand) => {
                CanMessage::StopLevitationCommand
            }
            MessageIdentifier::Event(EventId::LevitationStarted) => {
                CanMessage::LevitationStarted { board }
            }
            MessageIdentifier::Event(EventId::LevitationSystemsReady) => {
                CanMessage::LevitationSystemsReady
            }
            MessageIdentifier::Event(EventId::LevitationStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::TwoU16([current_ma, airgap_μm]) => CanMessage::LevitationStatus {
                        board,
                        current_ma: Current(current_ma),
                        airgap_μm: Airgap(airgap_μm),
                    },
                    _ => panic!("Invalid CanData for LevitationStatus"),
                }
            }

            MessageIdentifier::Event(EventId::LevitationStopped) => {
                CanMessage::LevitationStopped { board }
            }
            MessageIdentifier::Event(EventId::LevitationFailed) => {
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
            MessageIdentifier::Event(EventId::UnclampBrakesCommand) => {
                CanMessage::UnclampBrakesCommand
            }
            MessageIdentifier::Event(EventId::ClampBrakesCommand) => CanMessage::ClampBrakesCommand,
            MessageIdentifier::Event(EventId::RetractLateralSuspensionCommand) => {
                CanMessage::RetractLateralSuspensionCommand
            }
            MessageIdentifier::Event(EventId::ExtendLateralSuspensionCommand) => {
                CanMessage::ExtendLateralSuspensionCommand
            }
            MessageIdentifier::Event(EventId::BrakesClamped) => CanMessage::BrakesClamped { board },
            MessageIdentifier::Event(EventId::BrakesUnclamped) => {
                CanMessage::BrakesUnclamped { board }
            }
            MessageIdentifier::Event(EventId::LateralSuspensionRetracted) => {
                CanMessage::LateralSuspensionRetracted { board }
            }
            MessageIdentifier::Event(EventId::LateralSuspensionExtended) => {
                CanMessage::LateralSuspensionExtended { board }
            }
            MessageIdentifier::Event(EventId::DynamicsStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(pressure) => CanMessage::DynamicsStatus {
                        board,
                        actuator_pressure_bar: Pressure(pressure),
                    },
                    _ => panic!("Invalid CanData for DynamicsStatus"),
                }
            }

            // Propulsion
            MessageIdentifier::Event(EventId::StartPropulsionAccelerationCommand) => {
                CanMessage::StartPropulsionAccelerationCommand
            }
            MessageIdentifier::Event(EventId::StartPropulsionBrakingCommand) => {
                CanMessage::StartPropulsionBrakingCommand
            }
            MessageIdentifier::Event(EventId::PropulsionAccelerationStarted) => {
                CanMessage::PropulsionAccelerationStarted
            }
            MessageIdentifier::Event(EventId::PropulsionBrakingStarted) => {
                CanMessage::PropulsionBrakingStarted
            }
            MessageIdentifier::Event(EventId::PropulsionStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::PropulsionStatus {
                        current_ma,
                        velocity_kmh,
                        temperature_c,
                        voltage_cv,
                    } => CanMessage::PropulsionStatus {
                        current_ma: Current(current_ma),
                        velocity_kmh: Velocity(velocity_kmh),
                        temperature_c: Temperature(temperature_c),
                        voltage_cv: Voltage(voltage_cv),
                    },
                    _ => panic!("Invalid CanData for PropulsionStatus"),
                }
            }
            MessageIdentifier::Event(EventId::PropulsionForce) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(force) => CanMessage::PropulsionForce {
                        force_n: Force(force),
                    },
                    _ => panic!("Invalid CanData for PropulsionForce"),
                }
            }
            MessageIdentifier::Event(EventId::PropulsionFailed) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U8(reason_u8) => CanMessage::PropulsionFailed {
                        board,
                        reason: Reason::try_from(reason_u8).unwrap(),
                    },
                    _ => panic!("Invalid CanData for PropulsionFailed"),
                }
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
