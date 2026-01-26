use mc_logic::{decode_position_target_mm, decode_speed_accel_mmps, NavKinematics};

// Helpers to build little-endian i32 payloads.
fn pack_i32s(a: i32, b: i32) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[..4].copy_from_slice(&a.to_le_bytes());
    out[4..].copy_from_slice(&b.to_le_bytes());
    out
}

#[test]
fn nav_speed_accel_message_is_parsed() {
    // Simulate NAV sending speed=3.5 m/s, accel=-1.25 m/s^2
    let payload = pack_i32s(3500, -1250); // mm/s and mm/s^2
    let (speed, accel) = decode_speed_accel_mmps(&payload).expect("parse speed/accel");
    assert!((speed - 3.5).abs() < 1e-6);
    assert!((accel + 1.25).abs() < 1e-6);
}

#[test]
fn nav_position_target_message_is_parsed() {
    // Simulate NAV sending position=42.0 m, target=100.5 m
    let payload = pack_i32s(42_000, 100_500); // mm
    let (pos, target) = decode_position_target_mm(&payload).expect("parse pos/target");
    assert!((pos - 42.0).abs() < 1e-6);
    assert!((target - 100.5).abs() < 1e-6);
}

#[test]
fn kinematics_merge_keeps_existing_when_update_missing() {
    let base = NavKinematics {
        position_m: Some(10.0),
        velocity_mps: Some(5.0),
        accel_mps2: Some(0.2),
        target_position_m: Some(150.0),
    };
    let update = NavKinematics {
        position_m: None,
        velocity_mps: Some(4.5),
        accel_mps2: None,
        target_position_m: None,
    };
    let merged = base.merge(update);
    assert_eq!(merged.position_m, Some(10.0)); // kept old
    assert_eq!(merged.velocity_mps, Some(4.5)); // took new
    assert_eq!(merged.accel_mps2, Some(0.2)); // kept old
    assert_eq!(merged.target_position_m, Some(150.0)); // kept old
}

#[test]
fn kinematics_merge_overwrites_when_present() {
    let base = NavKinematics {
        position_m: Some(1.0),
        velocity_mps: Some(2.0),
        accel_mps2: Some(3.0),
        target_position_m: Some(4.0),
    };
    let update = NavKinematics {
        position_m: Some(10.0),
        velocity_mps: Some(20.0),
        accel_mps2: Some(30.0),
        target_position_m: Some(40.0),
    };
    let merged = base.merge(update);
    assert_eq!(merged.position_m, Some(10.0));
    assert_eq!(merged.velocity_mps, Some(20.0));
    assert_eq!(merged.accel_mps2, Some(30.0));
    assert_eq!(merged.target_position_m, Some(40.0));
}
