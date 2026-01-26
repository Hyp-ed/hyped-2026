#![cfg_attr(not(test), no_std)]

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct NavKinematics {
    pub position_m: Option<f32>,
    pub velocity_mps: Option<f32>,
    pub accel_mps2: Option<f32>,
    pub target_position_m: Option<f32>,
}

impl NavKinematics {
    pub fn merge(self, update: NavKinematics) -> NavKinematics {
        NavKinematics {
            position_m: update.position_m.or(self.position_m),
            velocity_mps: update.velocity_mps.or(self.velocity_mps),
            accel_mps2: update.accel_mps2.or(self.accel_mps2),
            target_position_m: update.target_position_m.or(self.target_position_m),
        }
    }
}

pub fn decode_speed_accel_mmps(data: &[u8]) -> Option<(f32, f32)> {
    if data.len() < 8 { return None; }
    let speed_mmps = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let accel_mmps2 = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    Some((speed_mmps as f32 / 1000.0, accel_mmps2 as f32 / 1000.0))
}

pub fn decode_position_target_mm(data: &[u8]) -> Option<(f32, f32)> {
    if data.len() < 8 { return None; }
    let pos_mm = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let target_mm = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    Some((pos_mm as f32 / 1000.0, target_mm as f32 / 1000.0))
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn decode_speed_and_accel_in_mmps() {
        // 1.234 m/s -> 1234 mm/s,  -0.500 m/s^2 -> -500 mm/s^2
        let bytes = 1234i32.to_le_bytes()
            .into_iter()
            .chain((-500i32).to_le_bytes())
            .collect::<std::vec::Vec<u8>>();

        let (speed, accel) = decode_speed_accel_mmps(&bytes).expect("should parse");
        assert!((speed - 1.234).abs() < 0.0001);
        assert!((accel + 0.5).abs() < 0.0001);
    }

    #[test]
    fn decode_position_and_target_in_mm() {
        // 12.345 m -> 12345 mm, 100.000 m -> 100000 mm
        let bytes = 12345i32.to_le_bytes()
            .into_iter()
            .chain(100_000i32.to_le_bytes())
            .collect::<std::vec::Vec<u8>>();

        let (pos, target) = decode_position_target_mm(&bytes).expect("should parse");
        assert!((pos - 12.345).abs() < 0.0001);
        assert!((target - 100.0).abs() < 0.0001);
    }

    #[test]
    fn nav_kinematics_merge_prefers_new_values() {
        let base = NavKinematics {
            position_m: Some(1.0),
            velocity_mps: Some(2.0),
            accel_mps2: None,
            target_position_m: Some(3.0),
        };
        let update = NavKinematics {
            position_m: Some(10.0),
            velocity_mps: None,
            accel_mps2: Some(-1.0),
            target_position_m: None,
        };
        let merged = base.merge(update);
        assert_eq!(merged.position_m, Some(10.0));
        assert_eq!(merged.velocity_mps, Some(2.0));
        assert_eq!(merged.accel_mps2, Some(-1.0));
        assert_eq!(merged.target_position_m, Some(3.0));
    }
}
