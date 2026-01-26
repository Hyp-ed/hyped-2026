use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
pub use mc_logic::{decode_position_target_mm, decode_speed_accel_mmps, NavKinematics};

pub static NAV_KINEMATICS: Channel<CriticalSectionRawMutex, NavKinematics, 5> = Channel::new();
