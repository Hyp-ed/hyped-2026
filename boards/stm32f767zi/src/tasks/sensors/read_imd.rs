use crate::tasks::can::{receive::INCOMING_IMD_MSGS, send::CAN_SEND};
use defmt_rtt as _;
use embassy_time::Duration;
use hyped_communications::messages::CanMessage;
use hyped_sensors::imd::is_frame_ok;

/// To be used on telemetry for now
#[embassy_executor::task]
pub async fn read_imd() {
    let rx = INCOMING_IMD_MSGS.receiver();
    let timeout = Duration::from_millis(1000);

    while let Ok(frame) = embassy_time::with_timeout(timeout, rx.receive()).await {
        defmt::debug!("Received IMD frame: {}", frame);

        if !is_frame_ok(frame) {
            defmt::error!("IMD Error: {:?}", frame.warnings_and_alarms);
            break;
        }

        CAN_SEND
            .send(CanMessage::MeasurementReading(
                hyped_communications::measurements::MeasurementReading {
                    reading: hyped_communications::data::CanData::U16(frame.corrected),
                    board: hyped_communications::boards::Board::Telemetry,
                    measurement_id: hyped_core::config::MeasurementId::ImdIsoCorrected,
                },
            ))
            .await;
    }

    CAN_SEND
        .send(CanMessage::Emergency(
            hyped_communications::boards::Board::Telemetry,
            hyped_communications::events::Reason::IMD,
        ))
        .await;
}
