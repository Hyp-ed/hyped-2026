#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, defmt::Format)]
pub enum Board {
    Telemetry = 0,
    Navigation = 1,
    Test = 3,
    TemperatureTester = 4,
    KeyenceTester = 5,
    StateMachineTester = 6,
    Mqtt = 7,
    MotorControl = 8,
    Pneumatics = 9,
    HighPower = 10,
}

impl From<Board> for u8 {
    fn from(board: Board) -> Self {
        board as u8
    }
}

impl TryFrom<u8> for Board {
    type Error = &'static str;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Board::Telemetry),
            1 => Ok(Board::Navigation),
            3 => Ok(Board::Test),
            4 => Ok(Board::TemperatureTester),
            5 => Ok(Board::KeyenceTester),
            6 => Ok(Board::StateMachineTester),
            7 => Ok(Board::Mqtt),
            9 => Ok(Board::Pneumatics),
            10 => Ok(Board::HighPower),
            8 => Ok(Board::MotorControl),
            _ => Err("Invalid Board index"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_conversion() {
        assert_eq!(
            Board::Telemetry,
            Board::try_from(Board::Telemetry as u8).unwrap()
        );
        assert_eq!(
            Board::Navigation,
            Board::try_from(Board::Navigation as u8).unwrap()
        );
        assert_eq!(
            Board::Pneumatics,
            Board::try_from(Board::Pneumatics as u8).unwrap()
        );
        assert_eq!(Board::Test, Board::try_from(Board::Test as u8).unwrap());
        assert_eq!(
            Board::TemperatureTester,
            Board::try_from(Board::TemperatureTester as u8).unwrap()
        );
        assert_eq!(
            Board::KeyenceTester,
            Board::try_from(Board::KeyenceTester as u8).unwrap()
        );
        assert_eq!(
            Board::StateMachineTester,
            Board::try_from(Board::StateMachineTester as u8).unwrap()
        );
        assert_eq!(Board::Mqtt, Board::try_from(Board::Mqtt as u8).unwrap());
        assert_eq!(u8::from(Board::Pneumatics), 9);
        assert_eq!(
            Board::HighPower,
            Board::try_from(Board::HighPower as u8).unwrap()
        );
        assert_eq!(u8::from(Board::HighPower), 10);

        assert_eq!(Board::try_from(2), Err("Invalid Board index"));
        assert_eq!(Board::try_from(11), Err("Invalid Board index"));

        assert_eq!(
            Board::MotorControl,
            Board::try_from(Board::MotorControl as u8).unwrap()
        );
    }
}
