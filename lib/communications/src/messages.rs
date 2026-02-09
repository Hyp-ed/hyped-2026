use hyped_can::HypedCanFrame;

use crate::{boards::Board, emergency::Reason};
use hyped_core::types::{Airgap, Current, Force, Pressure, Temperature, Velocity, Voltage};

use super::{
    can_id::CanId,
    data::{CanData, CanDataType},
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    message_identifier::{EventId, MessageIdentifier},
};

#[derive(PartialEq, Debug, Clone, defmt::Format)]
pub enum CanMessage {
    MeasurementReading(MeasurementReading),
    Heartbeat(Heartbeat),
    Emergency(Board, Reason),

    // Calibration
    StartCalibrationCommand,
    CalibrationComplete {
        from: Board,
    },

    // Electronics
    StartPrechargeCommand,
    StartDischargeCommand,
    PrechargeStarted,
    DischargeStarted,
    PrechargeComplete,
    DischargeComplete,
    VoltageStatus {
        voltage: Voltage,
    },
    PrechargeVoltageOK,
    DischargeVoltageOK,

    // Relays
    ShutdownCircuitryRelayOpen,
    ShutdownCircuitryRelayClosed,
    BatteryPrechargeRelayOpen,
    BatteryPrechargeRelayClosed,
    MotorControllerRelayOpen,
    MotorControllerRelayClosed,
    DischargeRelayOpen,
    DischargeRelayClosed,

    // Levitation
    StartLevitationCommand,
    StopLevitationCommand,
    LevitationSystemsReady,
    LevitationStarted {
        from: Board,
    },
    LevitationStopped {
        from: Board,
    },
    LevitationStatus {
        from: Board,
        current_ma: Current,
        airgap_μm: Airgap,
    },
    LevitationStable,

    // Navigation
    EndOfTrackBrake,

    // Dynamics
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,

    BrakesClamped {
        from: Board,
    },
    BrakesUnclamped {
        from: Board,
    },
    LateralSuspensionRetracted {
        from: Board,
    },
    LateralSuspensionExtended {
        from: Board,
    },
    DynamicsStatus {
        from: Board,
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
            CanMessage::Emergency(from, reason) => {
                let can_id = CanId::new(from, CanDataType::Emergency, MessageIdentifier::Emergency);
                HypedCanFrame::new(can_id.into(), CanData::Emergency(reason).into())
            }

            // Calibration
            CanMessage::StartCalibrationCommand => {
                let can_id: CanId = CanId::new_high_priority(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::StartCalibrationCommand),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::CalibrationComplete { from } => {
                let can_id: CanId = CanId::new(
                    from,
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
            CanMessage::PrechargeStarted => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PrechargeStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeStarted => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PrechargeComplete => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PrechargeComplete),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeComplete => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeComplete),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::VoltageStatus { voltage } => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U16,
                    MessageIdentifier::Event(EventId::VoltageStatus),
                );
                let data = CanData::U16(voltage.0).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::PrechargeVoltageOK => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PrechargeVoltageOK),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeVoltageOK => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeVoltageOK),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            // Relays
            CanMessage::ShutdownCircuitryRelayOpen => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::ShutdownCircuitryRelayOpen),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::ShutdownCircuitryRelayClosed => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::ShutdownCircuitryRelayClosed),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::BatteryPrechargeRelayOpen => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BatteryPrechargeRelayOpen),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::BatteryPrechargeRelayClosed => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BatteryPrechargeRelayClosed),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::MotorControllerRelayOpen => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::MotorControllerRelayOpen),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::MotorControllerRelayClosed => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::MotorControllerRelayClosed),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeRelayOpen => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeRelayOpen),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DischargeRelayClosed => {
                let can_id = CanId::new(
                    Board::Telemetry,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::DischargeRelayClosed),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
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
            CanMessage::LevitationStarted { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStopped { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationStopped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LevitationStatus {
                from,
                current_ma,
                airgap_μm,
            } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::TwoU16,
                    MessageIdentifier::Event(EventId::LevitationStatus),
                );
                let data: [u8; 8] = CanData::TwoU16([current_ma.0, airgap_μm.0]).into();
                HypedCanFrame::new(can_id.into(), data)
            }
            CanMessage::LevitationStable => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, //TODO: Placeholder, replace with real board later
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LevitationStable),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }

            // Navigation
            CanMessage::EndOfTrackBrake => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, //TODO: Placeholder, replace with real board later
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::EndOfTrackBrake),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
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

            CanMessage::BrakesClamped { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BrakesClamped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::BrakesUnclamped { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::BrakesUnclamped),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionExtended { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LateralSuspensionExtended),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::LateralSuspensionRetracted { from } => {
                let can_id: CanId = CanId::new(
                    from,
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::LateralSuspensionRetracted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::DynamicsStatus {
                from,
                actuator_pressure_bar,
            } => {
                let can_id: CanId = CanId::new(
                    from,
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
                    Board::Telemetry, //TODO: Placeholder, replace with real board later
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
                    Board::Telemetry, //TODO: Placeholder, replace with real board later
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PropulsionAccelerationStarted),
                );
                HypedCanFrame::new(can_id.into(), [0u8; 8])
            }
            CanMessage::PropulsionBrakingStarted => {
                let can_id: CanId = CanId::new(
                    Board::Telemetry, //TODO: Placeholder, replace with real board later
                    CanDataType::U32,
                    MessageIdentifier::Event(EventId::PropulsionBrakingStarted),
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
        let from = can_id.board;

        match message_identifier {
            // Existing
            MessageIdentifier::Measurement(measurement_id) => {
                let reading: CanData = frame.data.into();
                let measurement_reading = MeasurementReading {
                    reading,
                    board: from,
                    measurement_id,
                };
                CanMessage::MeasurementReading(measurement_reading)
            }
            MessageIdentifier::Heartbeat => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Heartbeat(to) => {
                        let heartbeat = Heartbeat::new(to, from);
                        CanMessage::Heartbeat(heartbeat)
                    }
                    _ => panic!("Invalid CanData for Heartbeat"),
                }
            }
            MessageIdentifier::Emergency => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Emergency(reason) => CanMessage::Emergency(from, reason),
                    _ => panic!("Invalid CanData for Emergency"),
                }
            }

            // Calibration
            MessageIdentifier::Event(EventId::StartCalibrationCommand) => {
                CanMessage::StartCalibrationCommand
            }
            MessageIdentifier::Event(EventId::CalibrationComplete) => {
                CanMessage::CalibrationComplete { from }
            }

            // Electronics
            MessageIdentifier::Event(EventId::StartPrechargeCommand) => {
                CanMessage::StartPrechargeCommand
            }
            MessageIdentifier::Event(EventId::StartDischargeCommand) => {
                CanMessage::StartDischargeCommand
            }
            MessageIdentifier::Event(EventId::PrechargeStarted) => CanMessage::PrechargeStarted,
            MessageIdentifier::Event(EventId::DischargeStarted) => CanMessage::DischargeStarted,
            MessageIdentifier::Event(EventId::PrechargeComplete) => CanMessage::PrechargeComplete,
            MessageIdentifier::Event(EventId::DischargeComplete) => CanMessage::DischargeComplete,
            MessageIdentifier::Event(EventId::VoltageStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(voltage) => CanMessage::VoltageStatus {
                        voltage: Voltage(voltage),
                    },
                    _ => panic!("Invalid CanData for VoltageStatus"),
                }
            }
            MessageIdentifier::Event(EventId::PrechargeVoltageOK) => {
                CanMessage::PrechargeVoltageOK
            }
            MessageIdentifier::Event(EventId::DischargeVoltageOK) => {
                CanMessage::DischargeVoltageOK
            }

            // Relays
            MessageIdentifier::Event(EventId::ShutdownCircuitryRelayOpen) => {
                CanMessage::ShutdownCircuitryRelayOpen
            }
            MessageIdentifier::Event(EventId::ShutdownCircuitryRelayClosed) => {
                CanMessage::ShutdownCircuitryRelayClosed
            }
            MessageIdentifier::Event(EventId::BatteryPrechargeRelayOpen) => {
                CanMessage::BatteryPrechargeRelayOpen
            }
            MessageIdentifier::Event(EventId::BatteryPrechargeRelayClosed) => {
                CanMessage::BatteryPrechargeRelayClosed
            }
            MessageIdentifier::Event(EventId::MotorControllerRelayOpen) => {
                CanMessage::MotorControllerRelayOpen
            }
            MessageIdentifier::Event(EventId::MotorControllerRelayClosed) => {
                CanMessage::MotorControllerRelayClosed
            }
            MessageIdentifier::Event(EventId::DischargeRelayOpen) => {
                CanMessage::DischargeRelayOpen
            }
            MessageIdentifier::Event(EventId::DischargeRelayClosed) => {
                CanMessage::DischargeRelayClosed
            }

            // Levitation
            MessageIdentifier::Event(EventId::StartLevitationCommand) => {
                CanMessage::StartLevitationCommand
            }
            MessageIdentifier::Event(EventId::StopLevitationCommand) => {
                CanMessage::StopLevitationCommand
            }
            MessageIdentifier::Event(EventId::LevitationStarted) => {
                CanMessage::LevitationStarted { from }
            }
            MessageIdentifier::Event(EventId::LevitationSystemsReady) => {
                CanMessage::LevitationSystemsReady
            }
            MessageIdentifier::Event(EventId::LevitationStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::TwoU16([current_ma, airgap_μm]) => CanMessage::LevitationStatus {
                        from,
                        current_ma: Current(current_ma),
                        airgap_μm: Airgap(airgap_μm),
                    },
                    _ => panic!("Invalid CanData for LevitationStatus"),
                }
            }

            MessageIdentifier::Event(EventId::LevitationStopped) => {
                CanMessage::LevitationStopped { from }
            }
            MessageIdentifier::Event(EventId::LevitationStable) => CanMessage::LevitationStable,

            // Navigation
            MessageIdentifier::Event(EventId::EndOfTrackBrake) => CanMessage::EndOfTrackBrake,

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
            MessageIdentifier::Event(EventId::BrakesClamped) => CanMessage::BrakesClamped { from },
            MessageIdentifier::Event(EventId::BrakesUnclamped) => {
                CanMessage::BrakesUnclamped { from }
            }
            MessageIdentifier::Event(EventId::LateralSuspensionRetracted) => {
                CanMessage::LateralSuspensionRetracted { from }
            }
            MessageIdentifier::Event(EventId::LateralSuspensionExtended) => {
                CanMessage::LateralSuspensionExtended { from }
            }
            MessageIdentifier::Event(EventId::DynamicsStatus) => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::U16(pressure) => CanMessage::DynamicsStatus {
                        from,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use hyped_can::HypedCanFrame;
    use hyped_core::config::MeasurementId;

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
