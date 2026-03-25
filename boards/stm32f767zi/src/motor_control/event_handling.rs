use embassy_time::{Duration, Timer};
use hyped_communications::boards::Board;
use hyped_communications::bus::EVENT_BUS;
use hyped_communications::events::Event;

use hyped_motors::can_open_processor::Messages;

use core::sync::atomic::{AtomicU32, Ordering};
use core::sync::atomic::AtomicBool;

static BRAKES_CLAMPED: AtomicBool = AtomicBool::new(true);
static PROPULSION_ACTIVE: AtomicBool = AtomicBool::new(false);

static CLAMP_CMD_COUNT: AtomicU32 = AtomicU32::new(0);
static UNCLAMP_CMD_COUNT: AtomicU32 = AtomicU32::new(0);
static ACCEL_CMD_COUNT: AtomicU32 = AtomicU32::new(0);
static BRAKE_CMD_COUNT: AtomicU32 = AtomicU32::new(0);

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
                let count = UNCLAMP_CMD_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                defmt::info!("UnclampBrakesCommand received (count={})", count);

                if BRAKES_CLAMPED.load(Ordering::Relaxed) {
                    defmt::info!("Unclamping brakes now");
                    release_brake_solenoid().await;
                    BRAKES_CLAMPED.store(false, Ordering::Relaxed);
                } else {
                    defmt::info!("Brakes already unclamped (idempotent)");
                }

                tx.send(Event::BrakesUnclamped { from: Board::MotorControl }).await;
            }

            Event::ClampBrakesCommand => {
                let count = CLAMP_CMD_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                defmt::info!("ClampBrakesCommand received (count={})", count);

                if !BRAKES_CLAMPED.load(Ordering::Relaxed) {
                    defmt::info!("Clamping brakes now");
                    engage_brake_solenoid().await;
                    BRAKES_CLAMPED.store(true, Ordering::Relaxed);
                } else {
                    defmt::info!("Brakes already clamped (idempotent)");
                }

                tx.send(Event::BrakesClamped { from: Board::MotorControl }).await;
            }

            // ===== Propulsion =====
            Event::StartPropulsionAccelerationCommand => {
                let count = ACCEL_CMD_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                defmt::info!("StartPropulsionAccelerationCommand received (count={})", count);

                if !PROPULSION_ACTIVE.load(Ordering::Relaxed) {
                    defmt::info!("Starting propulsion");
                    enqueue_canopen(Messages::StartDrive).await;
                    PROPULSION_ACTIVE.store(true, Ordering::Relaxed);
                } else {
                    defmt::info!("Propulsion already active (idempotent)");
                }

                tx.send(Event::PropulsionAccelerationStarted).await;
            }

            Event::StartPropulsionBrakingCommand => {
                let count = BRAKE_CMD_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                defmt::info!("StartPropulsionBrakingCommand received (count={})", count);

                if PROPULSION_ACTIVE.load(Ordering::Relaxed) {
                    defmt::info!("Stopping propulsion");
                    enqueue_canopen(Messages::QuickStop).await;
                    enqueue_canopen(Messages::Shutdown).await;
                    PROPULSION_ACTIVE.store(false, Ordering::Relaxed);
                } else {
                    defmt::info!("Propulsion already stopped (idempotent)");
                }

                tx.send(Event::PropulsionBrakingStarted).await;
            }

            _ => {
                // Ignore everything else
            }
        }
    }
}