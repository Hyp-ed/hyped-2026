use crate::{
    board_state::EMERGENCY,
    tasks::can::{receive::INCOMING_IMD_MSGS, send::CAN_SEND},
};
use defmt_rtt as _;
use embassy_time::Duration;
use hyped_communications::{
    boards::Board,
    data::CanData,
    events::Reason,
    measurements::MeasurementReading,
    messages::CanMessage,
};
use hyped_core::config::MeasurementId;
use hyped_sensors::imd::is_frame_ok;

/// To be used on telemetry for now
#[embassy_executor::task]
pub async fn read_imd() {
    let rx = INCOMING_IMD_MSGS.receiver();
    let timeout = Duration::from_millis(1000);

    loop {
        let frame = match embassy_time::with_timeout(timeout, rx.receive()).await {
            Ok(frame) => frame,
            Err(_) => {
                defmt::error!("No IMD frame received within {}ms", timeout.as_millis());
                break;
            }
        };

        if !is_frame_ok(frame) {
            defmt::error!("IMD Error: {:?}", frame.warnings_and_alarms);
            break;
        }

        defmt::debug!("Received IMD frame: {}", frame);

        CAN_SEND
            .send(CanMessage::MeasurementReading(MeasurementReading {
                reading: CanData::U16(frame.corrected),
                board: Board::Telemetry,
                measurement_id: MeasurementId::ImdIsoCorrected,
            }))
            .await;
    }

    EMERGENCY.sender().send(true);
    CAN_SEND
        .send(CanMessage::Emergency(Board::Telemetry, Reason::IMD))
        .await;
}
