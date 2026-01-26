use embassy_executor::task;

use crate::{
    engage_brake_solenoid, enqueue_canopen, release_brake_solenoid, BRAKE_MARGIN_M, FORCE_BRAKE,
    MAX_BRAKE_FORCE_N, POD_MASS_KG,
};
use mc_logic::NavKinematics;
use hyped_motors::can_open_processor::Messages;
use core::sync::atomic::Ordering;

// Brakes latch once engaged; no auto-release implemented yet.
#[task]
pub async fn braking_system_loop(
    mut nav_rx: embassy_sync::channel::Receiver<'static, NavKinematics>,
) {
    defmt::info!("Braking system active (mechanical brakes via GPIO)");

    let mut latest = NavKinematics::default();
    let mut brakes_engaged = false;
    let max_brake_decel = MAX_BRAKE_FORCE_N / POD_MASS_KG; // m/s^2

    loop {
        let update = nav_rx.receive().await;
        latest = latest.merge(update);

        if FORCE_BRAKE.load(Ordering::SeqCst) {
            if !brakes_engaged {
                defmt::warn!("Force brake engaged due to emergency");
                engage_brake_solenoid().await;
                brakes_engaged = true;
            }
            continue;
        }

        let (position, target, velocity) = match (
            latest.position_m,
            latest.target_position_m,
            latest.velocity_mps,
        ) {
            (Some(p), Some(t), Some(v)) => (p, t, v),
            _ => {
                defmt::debug!("Braking loop waiting for full kinematics (pos/target/vel)");
                continue;
            }
        };

        let distance_to_target = target - position;
        if distance_to_target <= 0.0 {
            if !brakes_engaged {
                defmt::warn!(
                    "Target passed or zero distance (pos={} m, target={} m); engaging brakes + QUICK_STOP",
                    position,
                    target
                );
                enqueue_canopen(Messages::QuickStop).await;
                engage_brake_solenoid().await;
                brakes_engaged = true;
            }
            continue;
        }

        // Required stopping distance under max braking decel (with margin)
        let stopping_distance = (velocity * velocity) / (2.0 * max_brake_decel) + BRAKE_MARGIN_M;

        if velocity > 0.1 && distance_to_target <= stopping_distance {
            if !brakes_engaged {
                defmt::info!(
                    "Engaging brakes: pos={} m, target={} m, dist={} m, vel={} m/s, stop_dist={} m",
                    position,
                    target,
                    distance_to_target,
                    velocity,
                    stopping_distance
                );
                // Tell motor controller to stop drive before clamping.
                enqueue_canopen(Messages::QuickStop).await;
                enqueue_canopen(Messages::Shutdown).await;
                // Brake solenoid clamps when driven low (per pneumatics::engage_brakes comments).
                engage_brake_solenoid().await;
                brakes_engaged = true;
            }
        } else {
            // Keep brakes released until within stopping distance.
            if brakes_engaged {
                defmt::info!("Brakes remain engaged (no auto-release implemented)");
            } else {
                release_brake_solenoid().await;
            }
        }
    }
}
