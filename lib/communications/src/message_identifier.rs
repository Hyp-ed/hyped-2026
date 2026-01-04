use hyped_core::config::MeasurementId;

#[derive(Debug, PartialEq, Clone)]
pub enum MessageIdentifier {
    Measurement(MeasurementId),
    Heartbeat,
    Emergency,

    // Calibration
    StartCalibrationCommand,
    CalibrationComplete,

    // Electronics
    StartPrechargeCommand,
    StartDischargeCommand,
    PrechargeStarted,
    DischargeStarted,
    PrechargeComplete,
    DischargeComplete,
    PrechargeFailed,

    // Levitation
    StartLevitationCommand,
    StopLevitationCommand,
    LevitationStarted,
    LevitationStopped,
    LevitationFailed,

    // Dynamics
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,
    BrakesClamped,
    BrakesUnclamped,
    LateralSuspensionRetracted,
    LateralSuspensionExtended,

    // Propulsion
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,
    PropulsionAccelerationStarted,
    PropulsionBrakingStarted,
}

// 12 bits
const MAX_MESSAGE_IDENTIFIER: u16 = 0xFFF;

const HEARTBEAT_ID: u16 = MAX_MESSAGE_IDENTIFIER - 1;
const EMERGENCY_ID: u16 = MAX_MESSAGE_IDENTIFIER - 2;

// Calibration
const START_CALIBRATION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 3;
const CALIBRATION_COMPLETE: u16 = MAX_MESSAGE_IDENTIFIER - 4;

// Electronics
const START_PRECHARGE_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 7;
const START_DISCHARGE_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 8;
const PRECHARGE_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 9;
const PRECHARGE_COMPLETE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 10;
const PRECHARGE_FAILED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 11;
const DISCHARGE_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 12;
const DISCHARGE_COMPLETE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 13;

// Levitation
const START_LEVITATION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 14;
const STOP_LEVITATION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 15;
const LEVITATION_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 16;
const LEVITATION_STOPPED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 17;
const LEVITATION_FAILED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 18;

// Dynamics
const UNCLAMP_BRAKES_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 19;
const CLAMP_BRAKES_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 20;
const BRAKES_CLAMPED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 21;
const BRAKES_UNCLAMPED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 22;
const RETRACT_LATERAL_SUSPENSION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 23;
const EXTEND_LATERAL_SUSPENSION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 24;
const LATERAL_SUSPENSION_RETRACTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 25;
const LATERAL_SUSPENSION_EXTENDED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 26;

// Propulsion
const START_PROPULSION_ACCELERATION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 27;
const START_PROPULSION_BRAKING_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 28;
const PROPULSION_ACCELERATION_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 29;
const PROPULSION_BRAKING_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 30;

impl From<MessageIdentifier> for u16 {
    fn from(val: MessageIdentifier) -> Self {
        match val {
            MessageIdentifier::Measurement(measurement_id) => measurement_id.into(),
            MessageIdentifier::Emergency => EMERGENCY_ID,
            MessageIdentifier::Heartbeat => HEARTBEAT_ID,

            // Calibration
            MessageIdentifier::StartCalibrationCommand => START_CALIBRATION_COMMAND_ID,
            MessageIdentifier::CalibrationComplete => CALIBRATION_COMPLETE,

            // Precharge/Discharge
            MessageIdentifier::StartPrechargeCommand => START_PRECHARGE_COMMAND_ID,
            MessageIdentifier::StartDischargeCommand => START_DISCHARGE_COMMAND_ID,
            MessageIdentifier::PrechargeStarted => PRECHARGE_STARTED_ID,
            MessageIdentifier::PrechargeComplete => PRECHARGE_COMPLETE_ID,
            MessageIdentifier::PrechargeFailed => PRECHARGE_FAILED_ID,
            MessageIdentifier::DischargeStarted => DISCHARGE_STARTED_ID,
            MessageIdentifier::DischargeComplete => DISCHARGE_COMPLETE_ID,

            // Levitation
            MessageIdentifier::StartLevitationCommand => START_LEVITATION_COMMAND_ID,
            MessageIdentifier::StopLevitationCommand => STOP_LEVITATION_COMMAND_ID,
            MessageIdentifier::LevitationStarted => LEVITATION_STARTED_ID,
            MessageIdentifier::LevitationStopped => LEVITATION_STOPPED_ID,
            MessageIdentifier::LevitationFailed => LEVITATION_FAILED_ID,

            // Dynamics
            MessageIdentifier::UnclampBrakesCommand => UNCLAMP_BRAKES_COMMAND_ID,
            MessageIdentifier::ClampBrakesCommand => CLAMP_BRAKES_COMMAND_ID,
            MessageIdentifier::BrakesClamped => BRAKES_CLAMPED_ID,
            MessageIdentifier::BrakesUnclamped => BRAKES_UNCLAMPED_ID,
            MessageIdentifier::RetractLateralSuspensionCommand => {
                RETRACT_LATERAL_SUSPENSION_COMMAND_ID
            }
            MessageIdentifier::ExtendLateralSuspensionCommand => {
                EXTEND_LATERAL_SUSPENSION_COMMAND_ID
            }
            MessageIdentifier::LateralSuspensionRetracted => LATERAL_SUSPENSION_RETRACTED_ID,
            MessageIdentifier::LateralSuspensionExtended => LATERAL_SUSPENSION_EXTENDED_ID,

            // Propulsion
            MessageIdentifier::StartPropulsionAccelerationCommand => {
                START_PROPULSION_ACCELERATION_COMMAND_ID
            }
            MessageIdentifier::StartPropulsionBrakingCommand => START_PROPULSION_BRAKING_COMMAND_ID,
            MessageIdentifier::PropulsionAccelerationStarted => PROPULSION_ACCELERATION_STARTED_ID,
            MessageIdentifier::PropulsionBrakingStarted => PROPULSION_BRAKING_STARTED_ID,
        }
    }
}

impl TryFrom<u16> for MessageIdentifier {
    type Error = &'static str;

    fn try_from(id: u16) -> Result<Self, Self::Error> {
        match id {
            HEARTBEAT_ID => Ok(MessageIdentifier::Heartbeat),
            EMERGENCY_ID => Ok(MessageIdentifier::Emergency),

            // Calibration
            START_CALIBRATION_COMMAND_ID => Ok(MessageIdentifier::StartCalibrationCommand),
            CALIBRATION_COMPLETE => Ok(MessageIdentifier::CalibrationComplete),

            // Electronics
            START_PRECHARGE_COMMAND_ID => Ok(MessageIdentifier::StartPrechargeCommand),
            START_DISCHARGE_COMMAND_ID => Ok(MessageIdentifier::StartDischargeCommand),
            PRECHARGE_STARTED_ID => Ok(MessageIdentifier::PrechargeStarted),
            PRECHARGE_COMPLETE_ID => Ok(MessageIdentifier::PrechargeComplete),
            PRECHARGE_FAILED_ID => Ok(MessageIdentifier::PrechargeFailed),
            DISCHARGE_STARTED_ID => Ok(MessageIdentifier::DischargeStarted),
            DISCHARGE_COMPLETE_ID => Ok(MessageIdentifier::DischargeComplete),

            // Levitation
            START_LEVITATION_COMMAND_ID => Ok(MessageIdentifier::StartLevitationCommand),
            STOP_LEVITATION_COMMAND_ID => Ok(MessageIdentifier::StopLevitationCommand),
            LEVITATION_STARTED_ID => Ok(MessageIdentifier::LevitationStarted),
            LEVITATION_STOPPED_ID => Ok(MessageIdentifier::LevitationStopped),
            LEVITATION_FAILED_ID => Ok(MessageIdentifier::LevitationFailed),

            // Dynamics
            UNCLAMP_BRAKES_COMMAND_ID => Ok(MessageIdentifier::UnclampBrakesCommand),
            CLAMP_BRAKES_COMMAND_ID => Ok(MessageIdentifier::ClampBrakesCommand),
            BRAKES_CLAMPED_ID => Ok(MessageIdentifier::BrakesClamped),
            RETRACT_LATERAL_SUSPENSION_COMMAND_ID => {
                Ok(MessageIdentifier::RetractLateralSuspensionCommand)
            }
            EXTEND_LATERAL_SUSPENSION_COMMAND_ID => {
                Ok(MessageIdentifier::ExtendLateralSuspensionCommand)
            }
            LATERAL_SUSPENSION_RETRACTED_ID => Ok(MessageIdentifier::LateralSuspensionRetracted),
            LATERAL_SUSPENSION_EXTENDED_ID => Ok(MessageIdentifier::LateralSuspensionExtended),

            // Propulsion
            START_PROPULSION_ACCELERATION_COMMAND_ID => {
                Ok(MessageIdentifier::StartPropulsionAccelerationCommand)
            }
            START_PROPULSION_BRAKING_COMMAND_ID => {
                Ok(MessageIdentifier::StartPropulsionBrakingCommand)
            }
            PROPULSION_ACCELERATION_STARTED_ID => {
                Ok(MessageIdentifier::PropulsionAccelerationStarted)
            }
            PROPULSION_BRAKING_STARTED_ID => Ok(MessageIdentifier::PropulsionBrakingStarted),

            // Fallback
            _ => match MeasurementId::try_from(id) {
                Ok(measurement_id) => Ok(MessageIdentifier::Measurement(measurement_id)),
                Err(e) => Err(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyped_core::config::MeasurementId;

    // #[test]
    // fn test_message_identifier_state_transition_command() {
    //     let message_identifier = MessageIdentifier::StateTransitionCommand;
    //     let encoded_message_identifier: u16 = message_identifier.clone().into();

    //     let decoded_message_identifier = MessageIdentifier::try_from(encoded_message_identifier)
    //         .expect("Failed to decode message identifier");
    //     assert_eq!(message_identifier, decoded_message_identifier);
    // }

    // #[test]
    // fn test_message_identifier_state_transition_request() {
    //     let message_identifier = MessageIdentifier::StateTransitionRequest;
    //     let encoded_message_identifier: u16 = message_identifier.clone().into();

    //     let decoded_message_identifier = MessageIdentifier::try_from(encoded_message_identifier)
    //         .expect("Failed to decode message identifier");
    //     assert_eq!(message_identifier, decoded_message_identifier);
    // }

    #[test]
    fn test_message_identifier_heartbeat() {
        let message_identifier = MessageIdentifier::Heartbeat;
        let encoded_message_identifier: u16 = message_identifier.clone().into();

        let decoded_message_identifier = MessageIdentifier::try_from(encoded_message_identifier)
            .expect("Failed to decode message identifier");
        assert_eq!(message_identifier, decoded_message_identifier);
    }

    #[test]
    fn test_message_identifier_measurement() {
        let message_identifier = MessageIdentifier::Measurement(MeasurementId::Thermistor1);
        let encoded_message_identifier: u16 = message_identifier.clone().into();

        let decoded_message_identifier = MessageIdentifier::try_from(encoded_message_identifier)
            .expect("Failed to decode message identifier");
        assert_eq!(message_identifier, decoded_message_identifier);
    }

    #[test]
    fn test_invalid_message_identifier() {
        assert!(MessageIdentifier::try_from(0xABCD).is_err());
    }
}
