use embassy_time::{Duration, Timer};
use hyped_communications::boards::Board;
use hyped_communications::bus::EVENT_BUS;
use hyped_communications::events::Event;

use hyped_motors::can_open_processor::Messages;

use crate::{enqueue_canopen, engage_brake_solenoid, release_brake_solenoid};

#[embassy_executor::task]
pub async fn motor_control_event_task() -> ! {
    let mut rx = EVENT_BUS.receiver();
    let tx = EVENT_BUS.sender();

    loop {
        let event = rx.receive().await;

        match event {
            // ===== Brakes =====
            Event::UnclampBrakesCommand => {
                // Do the action
                release_brake_solenoid().await;

                // Confirm completion (state machine waits for this)
                tx.send(Event::BrakesUnclamped { from: Board::MotorControl }).await;
            }

            Event::ClampBrakesCommand => {
                engage_brake_solenoid().await;

                tx.send(Event::BrakesClamped { from: Board::MotorControl }).await;
            }

            // ===== Propulsion =====
            Event::StartPropulsionAccelerationCommand => {
                // Immediately tell SM we started
                tx.send(Event::PropulsionAccelerationStarted).await;

                // Kick the motor controller
                enqueue_canopen(Messages::StartDrive).await;

                // Optional: if you have a "SwitchOn" / "Enable" sequence, do it here.
            }

            Event::StartPropulsionBrakingCommand => {
                tx.send(Event::PropulsionBrakingStarted).await;

                // If “braking” means motor quick stop + shutdown sequence:
                enqueue_canopen(Messages::QuickStop).await;
                enqueue_canopen(Messages::Shutdown).await;
            }

            _ => {
                // Ignore everything else
            }
        }
    }
}