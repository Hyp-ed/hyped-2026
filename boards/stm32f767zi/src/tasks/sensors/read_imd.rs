use crate::tasks::can::receive::INCOMING_IMD_MSGS;
use defmt_rtt as _;
use embassy_time::Duration;
use hyped_sensors::imd::is_frame_ok;

/// Test task that just reads the pressure from the low pressure sensor and prints it to the console
#[embassy_executor::task]
pub async fn read_imd() {
    let rx = INCOMING_IMD_MSGS.receiver();
    let timeout = Duration::from_millis(1000);

    while let Ok(frame) = embassy_time::with_timeout(timeout, rx.receive()).await {
        defmt::info!("Received IMD frame: {}", frame);

        if !is_frame_ok(frame) {
            break;
        }
    }

    // whatever emergency
}
