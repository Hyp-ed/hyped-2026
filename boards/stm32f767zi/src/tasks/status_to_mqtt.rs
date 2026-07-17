use core::{
    str::FromStr,
    sync::atomic::{AtomicU8, Ordering},
};

use embassy_time::{Duration, Ticker};
use heapless::String;
use hyped_core::{mqtt::MqttMessage, mqtt_topics::MqttTopic};

use super::mqtt::send::MQTT_SEND;

const STATUS_LOW_OR_FAULT: u8 = 0;
const STATUS_HIGH_OR_HEALTHY: u8 = 1;
const STATUS_UNKNOWN: u8 = 2;

static IMD_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);
static HVAL_RED_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);
static HVAL_GREEN_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);
static BRAKE_CLAMP_STATUS: AtomicU8 = AtomicU8::new(STATUS_UNKNOWN);

pub fn set_imd_status(healthy: bool) {
    IMD_STATUS.store(bool_to_status(healthy), Ordering::Release);
}

pub fn set_hval_status(red: Option<bool>, green: Option<bool>) {
    if let Some(active) = red {
        HVAL_RED_STATUS.store(bool_to_status(active), Ordering::Release);
    }
    if let Some(active) = green {
        HVAL_GREEN_STATUS.store(bool_to_status(active), Ordering::Release);
    }
}

pub fn set_brake_clamp_status(clamped: bool) {
    BRAKE_CLAMP_STATUS.store(bool_to_status(clamped), Ordering::Release);
}

fn bool_to_status(value: bool) -> u8 {
    if value {
        STATUS_HIGH_OR_HEALTHY
    } else {
        STATUS_LOW_OR_FAULT
    }
}

#[embassy_executor::task]
pub async fn publish_safety_statuses() {
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        publish_status(MqttTopic::ImdStatus, IMD_STATUS.load(Ordering::Acquire)).await;
        publish_status(
            MqttTopic::HvalRedStatus,
            HVAL_RED_STATUS.load(Ordering::Acquire),
        )
        .await;
        publish_status(
            MqttTopic::HvalGreenStatus,
            HVAL_GREEN_STATUS.load(Ordering::Acquire),
        )
        .await;
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
        STATUS_LOW_OR_FAULT => "0",
        STATUS_HIGH_OR_HEALTHY => "1",
        _ => "2",
    };

    MQTT_SEND
        .send(MqttMessage::new_retained(
            topic,
            String::from_str(payload).unwrap(),
        ))
        .await;
}
