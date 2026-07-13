use core::{
    str::FromStr,
    sync::atomic::{AtomicU8, Ordering},
};

use embassy_time::{Duration, Ticker};
use heapless::String;
use hyped_core::{mqtt::MqttMessage, mqtt_topics::MqttTopic};

use super::mqtt::send::MQTT_SEND;

const STATUS_FAULT_OR_UNCLAMPED: u8 = 0;
const STATUS_HEALTHY_OR_CLAMPED: u8 = 1;
const STATUS_UNKNOWN: u8 = 2;

static IMD_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);
static BRAKE_CLAMP_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);

pub fn set_imd_status(healthy: bool) {
    IMD_STATUS.store(
        if healthy {
            STATUS_HEALTHY_OR_CLAMPED
        } else {
            STATUS_FAULT_OR_UNCLAMPED
        },
        Ordering::Release,
    );
}

pub fn set_brake_clamp_status(clamped: bool) {
    BRAKE_CLAMP_STATUS.store(
        if clamped {
            STATUS_HEALTHY_OR_CLAMPED
        } else {
            STATUS_FAULT_OR_UNCLAMPED
        },
        Ordering::Release,
    );
}

#[embassy_executor::task]
pub async fn publish_safety_statuses() {
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        publish_status(MqttTopic::ImdStatus, IMD_STATUS.load(Ordering::Acquire)).await;
        publish_status(
            MqttTopic::BrakeClampStatus,
            BRAKE_CLAMP_STATUS.load(Ordering::Acquire),
        )
        .await;
        ticker.next().await;
    }
}

async fn publish_status(topic: MqttTopic, value: u8) {
    let payload = match value {
        STATUS_FAULT_OR_UNCLAMPED => "0",
        STATUS_HEALTHY_OR_CLAMPED => "1",
        _ => "2",
    };

    MQTT_SEND
        .send(MqttMessage::new_retained(
            topic,
            String::from_str(payload).unwrap(),
        ))
        .await;
}
