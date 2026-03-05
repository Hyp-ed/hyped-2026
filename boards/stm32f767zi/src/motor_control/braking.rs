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
async fn braking_system_loop(
    mut brake_solenoid_pin: Output<'static>,
    mut nav_rx: embassy_sync::channel::Receiver<'static, NavKinematics>,
) {
    defmt::info!("Braking system active (mechanical brakes via GPIO)");

    let mut latest = NavKinematics::default();
    let mut brakes_engaged = false;
    
    // Safety Parameters
    let max_brake_decel = MAX_BRAKE_FORCE_N / POD_MASS_KG; // m/s^2
    const DATA_TIMEOUT: Duration = Duration::from_millis(100); // Max time to wait for nav data

    brake_solenoid_pin.set_high(); // Unclamp brakes (Ensure brakes start at known state)

    loop {
        // Receive data with timeout
        // If not receiving data from Nav for longer than DATA_TIMEOUT, engage brakes
        match with_timeout(DATA_TIMEOUT, nav_rx.receive()).await {
            Ok(update) => {
                latest = latest.merge(update);
            },
            Err(_) => {
                if !brakes_engaged {
                    defmt::error!("CRITICAL: NAV data timeout! Engaging emergency brakes.")
                    enqueue_canopen(Messages::QuickStop).await;
                    brake_solenoid_pin.set_low();
                    brakes_engaged = true;
                }
                continue;
            }
        }

        let (position, target, velocity) = match (
            latest.position_m,
            latest.target_position_m,
            latest.velocity_mps,
        ) {
            (Some(p), Some(t), Some(v)) => (p, t, v),
            (_, _, Some(v)) if v > 1.0 => {
                if !brakes_engaged {
                    defmt::error!("CRITICAL: Lost Position/Target while moving (v={})! Engaging emergency brakes.", v)
                    enqueue_canopen(Messages::QuickStop).await;
                    brake_solenoid_pin.set_low();
                    brakes_engaged = true;
                }
                continue;
            }
            _ => {
                defmt::debug!("Braking loop waiting for full kinematics (pos/target/vel)");
                continue;
            }
        };

        // Distance travelled during reaction time (d = v * t)
        let reaction_dist = velocity * reaction_time_s;

        // Braking distance (d = v^2 / 2a)
        let physical_braking_distance = (velocity * velocity) / (2.0 * max_brake_decel);

        // Total required buffer distance to fully brake
        let required_stop_buffer_distance = reaction_dist + physical_braking_distance + SAFETY_MARGIN_M;
        // SAFETY_MARGIN_M from line 57 (to be implemented)

        let projected_stop_point = position + required_stop_buffer_distance;

        if velocity > 0.1 && projected_stop_point >= target {
            if !brakes_engaged {
                defmt::warn!(
                    "BRAKING POINT REACHED: Pos={:.2} m, Vel={:.2}, Target={:.2}, Proj_stop={:.2}",
                    position,
                    velocity,
                    target,
                    projected_stop_point
                );
                // Tell motor controller to stop drive before clamping.
                enqueue_canopen(Messages::QuickStop).await;
                enqueue_canopen(Messages::Shutdown).await;
                // Brake solenoid clamps when driven low (per pneumatics::engage_brakes comments).
                brake_solenoid_pin.set_low();
                brakes_engaged = true;
            }
        } else {
            if brakes_engaged {
                brake_solenoid_pin.set_high();
            }
        }
    }
}