use core::str::FromStr;
use heapless::String;

#[derive(PartialEq, Debug, defmt::Format, Clone, Copy)]
#[repr(u8)]
pub enum State {
    Idle = 0,
    Precharge = 1,
    ReadyForPropulsion = 2,
    Accelerate = 3,
    Brake = 4,
    Stopped = 5,
    Emergency = 6,
    SetupMotor = 7,
    Maintenance = 8,
    HvActive = 9,
    EnteringMaintenance = 10,
}

/// Mapping used to demonstrate the regulatory conditions without forcing operational states to
/// use the same names as the regulations.
#[derive(PartialEq, Debug, defmt::Format, Clone, Copy)]
pub enum RegulatoryState {
    Idle,
    Maintenance,
    HvActive,
    Demo,
    Emergency,
    Transitional,
}

impl From<State> for u8 {
    fn from(v: State) -> Self {
        v as u8
    }
}

impl State {
    pub fn regulatory_state(self) -> RegulatoryState {
        match self {
            State::Idle => RegulatoryState::Idle,
            State::Maintenance => RegulatoryState::Maintenance,
            State::HvActive => RegulatoryState::HvActive,
            State::Accelerate => RegulatoryState::Demo,
            State::Emergency => RegulatoryState::Emergency,
            State::EnteringMaintenance
            | State::SetupMotor
            | State::Precharge
            | State::ReadyForPropulsion
            | State::Brake
            | State::Stopped => RegulatoryState::Transitional,
        }
    }

    pub fn telemetry_state(self) -> &'static str {
        match self {
            State::Idle => "IDLE",
            State::Precharge => "PRECHARGE",
            State::ReadyForPropulsion => "READY_FOR_PROPULSION",
            State::Accelerate => "ACCELERATE",
            State::Brake => "BRAKE",
            State::Stopped => "STOPPED",
            State::Emergency => "EMERGENCY",
            State::SetupMotor => "SETUP_MOTOR",
            State::Maintenance => "MAINTENANCE",
            State::HvActive => "HV_ACTIVE",
            State::EnteringMaintenance => "ENTERING_MAINTENANCE",
        }
    }
}

impl TryFrom<u8> for State {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(State::Idle),
            0x01 => Ok(State::Precharge),
            0x02 => Ok(State::ReadyForPropulsion),
            0x03 => Ok(State::Accelerate),
            0x04 => Ok(State::Brake),
            0x05 => Ok(State::Stopped),
            0x06 => Ok(State::Emergency),
            0x07 => Ok(State::SetupMotor),
            0x08 => Ok(State::Maintenance),
            0x09 => Ok(State::HvActive),
            0x0A => Ok(State::EnteringMaintenance),
            _ => Err("Invalid state"),
        }
    }
}

impl From<State> for &str {
    fn from(val: State) -> Self {
        match val {
            State::Idle => "idle",
            State::Precharge => "precharge",
            State::ReadyForPropulsion => "ready_for_propulsion",
            State::Accelerate => "accelerate",
            State::Brake => "brake",
            State::Stopped => "stopped",
            State::Emergency => "emergency",
            State::SetupMotor => "setup_motor",
            State::Maintenance => "maintenance",
            State::HvActive => "hv_active",
            State::EnteringMaintenance => "entering_maintenance",
        }
    }
}

impl FromStr for State {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "idle" => Ok(State::Idle),
            "precharge" => Ok(State::Precharge),
            "ready_for_propulsion" => Ok(State::ReadyForPropulsion),
            "accelerate" => Ok(State::Accelerate),
            "brake" => Ok(State::Brake),
            "stopped" => Ok(State::Stopped),
            "emergency" => Ok(State::Emergency),
            "setup_motor" => Ok(State::SetupMotor),
            "maintenance" => Ok(State::Maintenance),
            "hv_active" => Ok(State::HvActive),
            "entering_maintenance" => Ok(State::EnteringMaintenance),
            _ => Err("Invalid state"),
        }
    }
}

impl From<State> for String<20> {
    fn from(val: State) -> Self {
        let mut s = String::new();
        s.push_str(val.into()).unwrap();
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_u8_round_trip() {
        let states = [
            State::Idle,
            State::Precharge,
            State::ReadyForPropulsion,
            State::Accelerate,
            State::Brake,
            State::Stopped,
            State::Emergency,
            State::SetupMotor,
            State::Maintenance,
            State::HvActive,
            State::EnteringMaintenance,
        ];

        for state in states {
            let encoded: u8 = state.into();
            let decoded = State::try_from(encoded).expect("decode state");
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn state_string_round_trip() {
        let states = [
            State::Idle,
            State::Precharge,
            State::ReadyForPropulsion,
            State::Accelerate,
            State::Brake,
            State::Stopped,
            State::Emergency,
            State::SetupMotor,
            State::Maintenance,
            State::HvActive,
            State::EnteringMaintenance,
        ];

        for state in states {
            let name: &str = state.into();
            let parsed = State::from_str(name).expect("parse state");
            assert_eq!(state, parsed);

            let heapless: String<20> = state.into();
            assert_eq!(name, heapless.as_str());
        }
    }

    #[test]
    fn invalid_state_rejected() {
        assert!(State::try_from(0xFF).is_err());
        assert!(State::from_str("unknown").is_err());
    }

    #[test]
    fn regulatory_state_mapping_is_explicit() {
        assert_eq!(State::Idle.regulatory_state(), RegulatoryState::Idle);
        assert_eq!(
            State::Maintenance.regulatory_state(),
            RegulatoryState::Maintenance
        );
        assert_eq!(
            State::HvActive.regulatory_state(),
            RegulatoryState::HvActive
        );
        assert_eq!(State::Accelerate.regulatory_state(), RegulatoryState::Demo);
        assert_eq!(
            State::Emergency.regulatory_state(),
            RegulatoryState::Emergency
        );
        assert_eq!(
            State::Precharge.regulatory_state(),
            RegulatoryState::Transitional
        );
    }
}
