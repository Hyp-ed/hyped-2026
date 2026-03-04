use core::fmt::Display;

use crate::emergency::Reason;

use super::boards::Board;

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum CanData {
    Bool(bool),
    TwoU16([u16; 2]),
    F32(f32),
    State(u8),
    U32(u32),
    Heartbeat(Board),
    Emergency(Reason),
    U8(u8),
    U16(u16),
    PropulsionStatus {
        current_ma: u16,
        velocity_kmh: u16,
        temperature_c: u8,
        voltage_cv: u16,
    },
}

impl Display for CanData {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CanData::Bool(b) => write!(formatter, "{b}"),
            CanData::TwoU16(u16s) => write!(formatter, "{u16s:?}"),
            CanData::F32(f) => write!(formatter, "{f}"),
            CanData::State(s) => write!(formatter, "{s}"),
            CanData::U32(u) => write!(formatter, "{u}"),
            CanData::Heartbeat(board) => write!(formatter, "{board:?}"),
            CanData::Emergency(reason) => write!(formatter, "{reason:?}"),
            CanData::U8(u) => write!(formatter, "{u}"),
            CanData::U16(u) => write!(formatter, "{u}"),
            CanData::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => {
                write!(formatter, "current={current_ma}mA, vel={velocity_kmh}km/h, temp={temperature_c}C, volt={voltage_cv}cV")
            }
        }
    }
}

impl From<CanData> for u8 {
    /// Gets the index of the CanData enum
    fn from(val: CanData) -> Self {
        match val {
            CanData::Bool(_) => 0,
            CanData::TwoU16(_) => 1,
            CanData::F32(_) => 2,
            CanData::State(_) => 3,
            CanData::U32(_) => 4,
            CanData::Heartbeat(_) => 5,
            CanData::Emergency(_) => 6,
            CanData::U8(_) => 7,
            CanData::U16(_) => 8,
            CanData::PropulsionStatus { .. } => 9,
        }
    }
}

impl From<u8> for CanData {
    /// Gets the CanData enum from the index
    fn from(index: u8) -> Self {
        match index {
            0 => CanData::Bool(false),
            1 => CanData::TwoU16([0, 0]),
            2 => CanData::F32(0.0),
            3 => CanData::State(0),
            4 => CanData::U32(0),
            5 => CanData::Heartbeat(Board::Test),
            6 => CanData::Emergency(Reason::Unknown),
            7 => CanData::U8(0),
            8 => CanData::U16(0),
            9 => CanData::PropulsionStatus {
                current_ma: 0,
                velocity_kmh: 0,
                temperature_c: 0,
                voltage_cv: 0,
            },
            _ => panic!("Invalid CanData index"),
        }
    }
}

impl From<CanData> for [u8; 8] {
    fn from(val: CanData) -> Self {
        match val {
            CanData::Bool(b) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1] = b as u8;
                data
            }
            CanData::TwoU16(u16s) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                let u16_bytes: [u8; 2] = u16s[0].to_le_bytes();
                data[1..3].copy_from_slice(&u16_bytes);

                let u16_bytes: [u8; 2] = u16s[1].to_le_bytes();
                data[3..5].copy_from_slice(&u16_bytes);

                data
            }
            CanData::F32(f) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                let f32_bytes: [u8; 4] = f.to_le_bytes();
                data[1..5].copy_from_slice(&f32_bytes);
                data
            }
            CanData::State(s) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1] = s;
                data
            }
            CanData::U32(u) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                let u32_bytes: [u8; 4] = u.to_le_bytes();
                data[1..5].copy_from_slice(&u32_bytes);
                data
            }
            CanData::Heartbeat(board) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1] = board.into();
                data
            }
            CanData::Emergency(reason) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1] = reason as u8;
                data
            }
            CanData::U8(u) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1] = u;
                data
            }
            CanData::U16(u) => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                let u16_bytes: [u8; 2] = u.to_le_bytes();
                data[1..3].copy_from_slice(&u16_bytes);
                data
            }
            CanData::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => {
                let mut data: [u8; 8] = [0; 8];
                data[0] = val.into();
                data[1..3].copy_from_slice(&current_ma.to_le_bytes());
                data[3..5].copy_from_slice(&velocity_kmh.to_le_bytes());
                data[5] = temperature_c;
                data[6..8].copy_from_slice(&voltage_cv.to_le_bytes());
                data
            }
        }
    }
}

impl From<[u8; 8]> for CanData {
    fn from(data: [u8; 8]) -> Self {
        let data_type: CanData = data[0].into();
        match data_type {
            CanData::Bool(_) => CanData::Bool(data[1] != 0),
            CanData::TwoU16(_) => {
                let mut u16_bytes: [u8; 2] = [0; 2];
                u16_bytes.copy_from_slice(&data[1..3]);
                let u16_1 = u16::from_le_bytes(u16_bytes);

                u16_bytes.copy_from_slice(&data[3..5]);
                let u16_2 = u16::from_le_bytes(u16_bytes);

                CanData::TwoU16([u16_1, u16_2])
            }
            CanData::F32(_) => {
                let mut f32_bytes: [u8; 4] = [0; 4];
                f32_bytes.copy_from_slice(&data[1..5]);
                let f = f32::from_le_bytes(f32_bytes);
                CanData::F32(f)
            }
            CanData::State(_) => CanData::State(data[1]),
            CanData::U32(_) => {
                let mut u32_bytes: [u8; 4] = [0; 4];
                u32_bytes.copy_from_slice(&data[1..5]);
                let u = u32::from_le_bytes(u32_bytes);
                CanData::U32(u)
            }
            CanData::Heartbeat(_) => CanData::Heartbeat(data[1].try_into().unwrap()),
            CanData::Emergency(_) => CanData::Emergency(data[1].try_into().unwrap()),
            CanData::U8(_) => CanData::U8(data[1]),
            CanData::U16(_) => {
                let mut u16_bytes: [u8; 2] = [0; 2];
                u16_bytes.copy_from_slice(&data[1..3]);
                let u = u16::from_le_bytes(u16_bytes);
                CanData::U16(u)
            }
            CanData::PropulsionStatus { .. } => {
                let current_ma = u16::from_le_bytes([data[1], data[2]]);
                let velocity_kmh = u16::from_le_bytes([data[3], data[4]]);
                let temperature_c = data[5];
                let voltage_cv = u16::from_le_bytes([data[6], data[7]]);
                CanData::PropulsionStatus {
                    current_ma,
                    velocity_kmh,
                    temperature_c,
                    voltage_cv,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
#[repr(u8)]
/// Enum representing the data type of the CAN message's data.
pub enum CanDataType {
    Bool = 0,
    TwoU16 = 1,
    F32 = 2,
    State = 3,
    U32 = 4,
    Heartbeat = 5,
    Emergency = 6,
    U8 = 7,
    U16 = 8,
    PropulsionStatus = 9,
}

impl From<CanDataType> for u8 {
    fn from(v: CanDataType) -> Self {
        v as u8
    }
}

impl TryFrom<u8> for CanDataType {
    type Error = &'static str;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(CanDataType::Bool),
            1 => Ok(CanDataType::TwoU16),
            2 => Ok(CanDataType::F32),
            3 => Ok(CanDataType::State),
            4 => Ok(CanDataType::U32),
            5 => Ok(CanDataType::Heartbeat),
            6 => Ok(CanDataType::Emergency),
            7 => Ok(CanDataType::U8),
            8 => Ok(CanDataType::U16),
            9 => Ok(CanDataType::PropulsionStatus),
            _ => Err("Invalid CanDataType index"),
        }
    }
}

impl From<CanData> for CanDataType {
    fn from(val: CanData) -> Self {
        match val {
            CanData::Bool(_) => CanDataType::Bool,
            CanData::TwoU16(_) => CanDataType::TwoU16,
            CanData::F32(_) => CanDataType::F32,
            CanData::State(_) => CanDataType::State,
            CanData::U32(_) => CanDataType::U32,
            CanData::Heartbeat(_) => CanDataType::Heartbeat,
            CanData::Emergency(_) => CanDataType::Emergency,
            CanData::U8(_) => CanDataType::U8,
            CanData::U16(_) => CanDataType::U16,
            CanData::PropulsionStatus { .. } => CanDataType::PropulsionStatus,
        }
    }
}

impl From<CanDataType> for CanData {
    fn from(data_type: CanDataType) -> Self {
        match data_type {
            CanDataType::Bool => CanData::Bool(false),
            CanDataType::TwoU16 => CanData::TwoU16([0, 0]),
            CanDataType::F32 => CanData::F32(0.0),
            CanDataType::State => CanData::State(0),
            CanDataType::U32 => CanData::U32(0),
            CanDataType::Heartbeat => CanData::Heartbeat(Board::Test),
            CanDataType::Emergency => CanData::Emergency(Reason::Unknown),
            CanDataType::U8 => CanData::U8(0),
            CanDataType::U16 => CanData::U16(0),
            CanDataType::PropulsionStatus => CanData::PropulsionStatus {
                current_ma: 0,
                velocity_kmh: 0,
                temperature_c: 0,
                voltage_cv: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(data: CanData) {
        let encoded: [u8; 8] = data.into();
        let decoded: CanData = encoded.into();
        assert_eq!(data, decoded);
    }

    #[test]
    fn can_data_round_trips_bytes() {
        round_trip(CanData::Bool(true));
        round_trip(CanData::TwoU16([12, 34]));
        round_trip(CanData::F32(12.5));
        round_trip(CanData::State(7));
        round_trip(CanData::U32(0xDEADBEEF));
        round_trip(CanData::Heartbeat(Board::Navigation));
        round_trip(CanData::Emergency(Reason::MissingHeartbeat));
    }

    #[test]
    fn can_data_type_round_trips() {
        let data_types = [
            CanDataType::Bool,
            CanDataType::TwoU16,
            CanDataType::F32,
            CanDataType::State,
            CanDataType::U32,
            CanDataType::Heartbeat,
            CanDataType::Emergency,
        ];

        for data_type in data_types {
            let encoded: u8 = data_type.into();
            let decoded = CanDataType::try_from(encoded).expect("decode CanDataType");
            assert_eq!(data_type, decoded);

            let default_data: CanData = data_type.into();
            assert_eq!(CanDataType::from(default_data), data_type);
        }
    }
}
