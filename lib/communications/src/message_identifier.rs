use hyped_core::config::MeasurementId;

#[derive(Debug, PartialEq, Clone)]
pub enum MessageIdentifier {
    Measurement(MeasurementId),
    Heartbeat,
    Emergency,
    Event(EventId),
}

#[derive(Debug, PartialEq, Clone)]
pub enum EventId {
    // Electronics
    StartPrechargeCommand,
    StartDischargeCommand,
    PrechargeStarted,
    DischargeStarted,
    PrechargeComplete,
    DischargeComplete,
    VoltageStatus,
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

    // Navigation
    EndOfTrackBrake,

    // Dynamics
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,
    BrakesClamped,
    BrakesUnclamped,
    LateralSuspensionRetracted,
    LateralSuspensionExtended,
    DynamicsStatus,

    // Propulsion
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,
    PropulsionAccelerationStarted,
    PropulsionBrakingStarted,
    PropulsionStatus,
    PropulsionForce,
}

// 12 bits
const MAX_MESSAGE_IDENTIFIER: u16 = 0xFFF;

const HEARTBEAT_ID: u16 = MAX_MESSAGE_IDENTIFIER - 1;
const EMERGENCY_ID: u16 = MAX_MESSAGE_IDENTIFIER - 2;

// Electronics
const START_PRECHARGE_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 7;
const START_DISCHARGE_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 8;
const PRECHARGE_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 9;
const PRECHARGE_COMPLETE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 10;
const DISCHARGE_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 12;
const DISCHARGE_COMPLETE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 13;

// Note: 3, 4, 11, 14-20 are free

// Navigation
const END_OF_TRACK_BRAKE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 38;

// Dynamics
const UNCLAMP_BRAKES_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 21;
const CLAMP_BRAKES_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 22;
const BRAKES_CLAMPED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 23;
const BRAKES_UNCLAMPED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 24;
const RETRACT_LATERAL_SUSPENSION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 25;
const EXTEND_LATERAL_SUSPENSION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 26;
const LATERAL_SUSPENSION_RETRACTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 27;
const LATERAL_SUSPENSION_EXTENDED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 28;
const DYNAMICS_STATUS_ID: u16 = MAX_MESSAGE_IDENTIFIER - 29;

// Propulsion
const START_PROPULSION_ACCELERATION_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 30;
const START_PROPULSION_BRAKING_COMMAND_ID: u16 = MAX_MESSAGE_IDENTIFIER - 31;
const PROPULSION_ACCELERATION_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 32;
const PROPULSION_BRAKING_STARTED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 33;
const PROPULSION_STATUS_ID: u16 = MAX_MESSAGE_IDENTIFIER - 34;
const PROPULSION_FORCE_ID: u16 = MAX_MESSAGE_IDENTIFIER - 35;

// Electronics (cont.)
const VOLTAGE_STATUS_ID: u16 = MAX_MESSAGE_IDENTIFIER - 39;
const PRECHARGE_VOLTAGE_OK_ID: u16 = MAX_MESSAGE_IDENTIFIER - 40;
const DISCHARGE_VOLTAGE_OK_ID: u16 = MAX_MESSAGE_IDENTIFIER - 41;

// Relays
const SHUTDOWN_CIRCUITRY_RELAY_OPEN_ID: u16 = MAX_MESSAGE_IDENTIFIER - 42;
const SHUTDOWN_CIRCUITRY_RELAY_CLOSED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 43;
const BATTERY_PRECHARGE_RELAY_OPEN_ID: u16 = MAX_MESSAGE_IDENTIFIER - 44;
const BATTERY_PRECHARGE_RELAY_CLOSED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 45;
const MOTOR_CONTROLLER_RELAY_OPEN_ID: u16 = MAX_MESSAGE_IDENTIFIER - 46;
const MOTOR_CONTROLLER_RELAY_CLOSED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 47;
const DISCHARGE_RELAY_OPEN_ID: u16 = MAX_MESSAGE_IDENTIFIER - 48;
const DISCHARGE_RELAY_CLOSED_ID: u16 = MAX_MESSAGE_IDENTIFIER - 49;

impl From<EventId> for u16 {
    fn from(val: EventId) -> Self {
        match val {
            // Electronics
            EventId::StartPrechargeCommand => START_PRECHARGE_COMMAND_ID,
            EventId::StartDischargeCommand => START_DISCHARGE_COMMAND_ID,
            EventId::PrechargeStarted => PRECHARGE_STARTED_ID,
            EventId::PrechargeComplete => PRECHARGE_COMPLETE_ID,
            EventId::DischargeStarted => DISCHARGE_STARTED_ID,
            EventId::DischargeComplete => DISCHARGE_COMPLETE_ID,

            // Navigation
            EventId::EndOfTrackBrake => END_OF_TRACK_BRAKE_ID,

            // Dynamics
            EventId::UnclampBrakesCommand => UNCLAMP_BRAKES_COMMAND_ID,
            EventId::ClampBrakesCommand => CLAMP_BRAKES_COMMAND_ID,
            EventId::BrakesClamped => BRAKES_CLAMPED_ID,
            EventId::BrakesUnclamped => BRAKES_UNCLAMPED_ID,
            EventId::RetractLateralSuspensionCommand => RETRACT_LATERAL_SUSPENSION_COMMAND_ID,
            EventId::ExtendLateralSuspensionCommand => EXTEND_LATERAL_SUSPENSION_COMMAND_ID,
            EventId::LateralSuspensionRetracted => LATERAL_SUSPENSION_RETRACTED_ID,
            EventId::LateralSuspensionExtended => LATERAL_SUSPENSION_EXTENDED_ID,
            EventId::DynamicsStatus => DYNAMICS_STATUS_ID,

            // Propulsion
            EventId::StartPropulsionAccelerationCommand => START_PROPULSION_ACCELERATION_COMMAND_ID,
            EventId::StartPropulsionBrakingCommand => START_PROPULSION_BRAKING_COMMAND_ID,
            EventId::PropulsionAccelerationStarted => PROPULSION_ACCELERATION_STARTED_ID,
            EventId::PropulsionBrakingStarted => PROPULSION_BRAKING_STARTED_ID,
            EventId::PropulsionStatus => PROPULSION_STATUS_ID,
            EventId::PropulsionForce => PROPULSION_FORCE_ID,

            // Electronics (cont.)
            EventId::VoltageStatus => VOLTAGE_STATUS_ID,
            EventId::PrechargeVoltageOK => PRECHARGE_VOLTAGE_OK_ID,
            EventId::DischargeVoltageOK => DISCHARGE_VOLTAGE_OK_ID,

            // Relays
            EventId::ShutdownCircuitryRelayOpen => SHUTDOWN_CIRCUITRY_RELAY_OPEN_ID,
            EventId::ShutdownCircuitryRelayClosed => SHUTDOWN_CIRCUITRY_RELAY_CLOSED_ID,
            EventId::BatteryPrechargeRelayOpen => BATTERY_PRECHARGE_RELAY_OPEN_ID,
            EventId::BatteryPrechargeRelayClosed => BATTERY_PRECHARGE_RELAY_CLOSED_ID,
            EventId::MotorControllerRelayOpen => MOTOR_CONTROLLER_RELAY_OPEN_ID,
            EventId::MotorControllerRelayClosed => MOTOR_CONTROLLER_RELAY_CLOSED_ID,
            EventId::DischargeRelayOpen => DISCHARGE_RELAY_OPEN_ID,
            EventId::DischargeRelayClosed => DISCHARGE_RELAY_CLOSED_ID,
        }
    }
}

impl TryFrom<u16> for EventId {
    type Error = &'static str;

    fn try_from(id: u16) -> Result<Self, Self::Error> {
        match id {
            // Electronics
            START_PRECHARGE_COMMAND_ID => Ok(EventId::StartPrechargeCommand),
            START_DISCHARGE_COMMAND_ID => Ok(EventId::StartDischargeCommand),
            PRECHARGE_STARTED_ID => Ok(EventId::PrechargeStarted),
            PRECHARGE_COMPLETE_ID => Ok(EventId::PrechargeComplete),
            DISCHARGE_STARTED_ID => Ok(EventId::DischargeStarted),
            DISCHARGE_COMPLETE_ID => Ok(EventId::DischargeComplete),

            // Navigation
            END_OF_TRACK_BRAKE_ID => Ok(EventId::EndOfTrackBrake),

            // Dynamics
            UNCLAMP_BRAKES_COMMAND_ID => Ok(EventId::UnclampBrakesCommand),
            CLAMP_BRAKES_COMMAND_ID => Ok(EventId::ClampBrakesCommand),
            BRAKES_CLAMPED_ID => Ok(EventId::BrakesClamped),
            BRAKES_UNCLAMPED_ID => Ok(EventId::BrakesUnclamped),
            RETRACT_LATERAL_SUSPENSION_COMMAND_ID => Ok(EventId::RetractLateralSuspensionCommand),
            EXTEND_LATERAL_SUSPENSION_COMMAND_ID => Ok(EventId::ExtendLateralSuspensionCommand),
            LATERAL_SUSPENSION_RETRACTED_ID => Ok(EventId::LateralSuspensionRetracted),
            LATERAL_SUSPENSION_EXTENDED_ID => Ok(EventId::LateralSuspensionExtended),
            DYNAMICS_STATUS_ID => Ok(EventId::DynamicsStatus),

            // Propulsion
            START_PROPULSION_ACCELERATION_COMMAND_ID => {
                Ok(EventId::StartPropulsionAccelerationCommand)
            }
            START_PROPULSION_BRAKING_COMMAND_ID => Ok(EventId::StartPropulsionBrakingCommand),
            PROPULSION_ACCELERATION_STARTED_ID => Ok(EventId::PropulsionAccelerationStarted),
            PROPULSION_BRAKING_STARTED_ID => Ok(EventId::PropulsionBrakingStarted),
            PROPULSION_STATUS_ID => Ok(EventId::PropulsionStatus),
            PROPULSION_FORCE_ID => Ok(EventId::PropulsionForce),

            // Electronics (cont.)
            VOLTAGE_STATUS_ID => Ok(EventId::VoltageStatus),
            PRECHARGE_VOLTAGE_OK_ID => Ok(EventId::PrechargeVoltageOK),
            DISCHARGE_VOLTAGE_OK_ID => Ok(EventId::DischargeVoltageOK),

            // Relays
            SHUTDOWN_CIRCUITRY_RELAY_OPEN_ID => Ok(EventId::ShutdownCircuitryRelayOpen),
            SHUTDOWN_CIRCUITRY_RELAY_CLOSED_ID => Ok(EventId::ShutdownCircuitryRelayClosed),
            BATTERY_PRECHARGE_RELAY_OPEN_ID => Ok(EventId::BatteryPrechargeRelayOpen),
            BATTERY_PRECHARGE_RELAY_CLOSED_ID => Ok(EventId::BatteryPrechargeRelayClosed),
            MOTOR_CONTROLLER_RELAY_OPEN_ID => Ok(EventId::MotorControllerRelayOpen),
            MOTOR_CONTROLLER_RELAY_CLOSED_ID => Ok(EventId::MotorControllerRelayClosed),
            DISCHARGE_RELAY_OPEN_ID => Ok(EventId::DischargeRelayOpen),
            DISCHARGE_RELAY_CLOSED_ID => Ok(EventId::DischargeRelayClosed),

            _ => Err("Invalid EventId"),
        }
    }
}

impl From<MessageIdentifier> for u16 {
    fn from(val: MessageIdentifier) -> Self {
        match val {
            MessageIdentifier::Measurement(measurement_id) => measurement_id.into(),
            MessageIdentifier::Emergency => EMERGENCY_ID,
            MessageIdentifier::Heartbeat => HEARTBEAT_ID,
            MessageIdentifier::Event(event_id) => event_id.into(),
        }
    }
}

impl TryFrom<u16> for MessageIdentifier {
    type Error = &'static str;

    fn try_from(id: u16) -> Result<Self, Self::Error> {
        match id {
            HEARTBEAT_ID => Ok(MessageIdentifier::Heartbeat),
            EMERGENCY_ID => Ok(MessageIdentifier::Emergency),

            _ => {
                if let Ok(event_id) = EventId::try_from(id) {
                    Ok(MessageIdentifier::Event(event_id))
                } else if let Ok(measurement_id) = MeasurementId::try_from(id) {
                    Ok(MessageIdentifier::Measurement(measurement_id))
                } else {
                    Err("Invalid MessageIdentifier")
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use hyped_core::config::MeasurementId;

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
