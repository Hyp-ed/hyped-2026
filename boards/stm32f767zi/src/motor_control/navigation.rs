use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

#[derive(Default, Clone, Copy, defmt::Format)]
pub struct NavKinematics {
    pub position_m: Option<f32>,
    pub target_position_m: Option<f32>,
    pub velocity_mps: Option<f32>,
    pub accel_mps2: Option<f32>,
}

impl NavKinematics {
    pub fn merge(self, other: NavKinematics) -> Self {
        Self {
            position_m: other.position_m.or(self.position_m),
            target_position_m: other.target_position_m.or(self.target_position_m),
            velocity_mps: other.velocity_mps.or(self.velocity_mps),
            accel_mps2: other.accel_mps2.or(self.accel_mps2),
        }
    }
}

pub static NAV_KINEMATICS: Channel<CriticalSectionRawMutex, NavKinematics, 5> = Channel::new();

pub fn decode_speed_accel_mmps(data: &[u8]) -> Option<(f32, f32)> {
    if data.len() < 8 {
        return None;
    }
    let speed_mmps = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let accel_mmps2 = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    Some((speed_mmps as f32 / 1000.0, accel_mmps2 as f32 / 1000.0))
}

pub fn decode_position_target_mm(data: &[u8]) -> Option<(f32, f32)> {
    if data.len() < 8 {
        return None;
    }
    let position_mm = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let target_mm = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    Some((position_mm as f32 / 1000.0, target_mm as f32 / 1000.0))
}
