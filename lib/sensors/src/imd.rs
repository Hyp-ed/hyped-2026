use bitfield::bitfield;
use defmt::Format;

bitfield! {
    #[derive(Copy, Clone, Format)]
    #[repr(C)]
    pub struct WarningsAndAlarms(u16);
    impl Debug;
    pub device_error,       _: 0;   // Internal IMD fault
    pub hv_pos_failure,     _: 1;   // HV+ not connected to IMD
    pub hv_neg_failure,     _: 2;   // HV- not connected to IMD
    pub earth_failure,      _: 3;   // Earth/chassis wire missing or broken
    pub iso_alarm,          _: 4;   // Isolation below error threshold (default 100kΩ)
    pub iso_warning,        _: 5;   // Isolation below warning threshold (default 500kΩ)
    pub iso_outdated,       _: 6;   // Measurement timed out, value stale
    pub unbalance_alarm,    _: 7;   // HV midpoint shifted, asymmetric fault
    pub undervoltage_alarm, _: 8;   // HV too low for reliable measurement
    pub unsafe_to_start,    _: 9;   // IMD says do not enable HV
    pub earthlift_open,     _: 10;  // Internal earth disconnector relay is open
}

#[derive(Copy, Clone, Debug, Format)]
#[repr(C)]
pub struct ImdFrame {
    pub corrected: u16,
    pub status: u8,
    pub measurement_counter: u8,
    pub warnings_and_alarms: WarningsAndAlarms,
    pub device_activity: u8,
    pub reserved: u8,
}

impl ImdFrame {
    pub fn from_data(data: &[u8]) -> Self {
        Self {
            corrected: u16::from_le_bytes([data[0], data[1]]),
            status: data[2],
            measurement_counter: data[3],
            warnings_and_alarms: WarningsAndAlarms(u16::from_le_bytes([data[4], data[5]])),
            device_activity: data[6],
            reserved: data[7],
        }
    }
}

pub fn is_frame_ok(frame: ImdFrame) -> bool {
    frame.warnings_and_alarms.0 == 0
}
