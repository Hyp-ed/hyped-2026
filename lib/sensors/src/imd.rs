use bytemuck::{Pod, Zeroable};
use defmt::Format;

#[derive(Zeroable, Pod, Copy, Clone, Debug, Format)]
#[repr(C)]
pub struct ImdFrame {
    pub corrected: u16,
    pub status: u8,
    pub measurement_counter: u8,
    pub warnings_and_alarms: u16,
    pub device_activity: u8,
    pub reserved: u8,
}

impl ImdFrame {
    pub fn from_data(data: &[u8]) -> Self {
        *bytemuck::from_bytes(data)
    }
}

pub fn is_frame_ok(frame: ImdFrame) -> bool {
    true
}
