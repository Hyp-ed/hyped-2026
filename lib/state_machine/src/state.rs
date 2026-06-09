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
}

impl From<State> for u8 {
    fn from(v: State) -> Self {
        v as u8
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
}
