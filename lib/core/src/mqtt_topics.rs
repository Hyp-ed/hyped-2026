// TODOLater: pod name

use crate::config::MeasurementId;
use core::str::FromStr;
use heapless::String;

pub const MQTT_MEASUREMENT_TOPIC_PREFIX: &str = "hyped/poddington/measurement/";

/// Enum representing all MQTT topics used by the pod
#[derive(Debug, defmt::Format, PartialEq, Eq)]
pub enum MqttTopic {
    Measurement(MeasurementId),
    State,
    StateRequest,
    Heartbeat,
    Logs,
    Debug,
    Test,
    LatencyRequest,
    LatencyResponse,
}

impl FromStr for MqttTopic {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hyped/poddington/state/state" => Ok(MqttTopic::State),
            "hyped/poddington/state/state_request" => Ok(MqttTopic::StateRequest),
            "hyped/poddington/heartbeat" => Ok(MqttTopic::Heartbeat),
            "hyped/poddington/logs" => Ok(MqttTopic::Logs),
            "hyped/poddington/latency/request" => Ok(MqttTopic::LatencyRequest),
            "hyped/poddington/latency/response" => Ok(MqttTopic::LatencyResponse),
            "debug" => Ok(MqttTopic::Debug),
            "test" => Ok(MqttTopic::Test),
            _ => {
                if s.starts_with(MQTT_MEASUREMENT_TOPIC_PREFIX) {
                    let measurement_id_string = &s[MQTT_MEASUREMENT_TOPIC_PREFIX.len()..s.len()];
                    let measurement_id = measurement_id_string.into();
                    Ok(MqttTopic::Measurement(measurement_id))
                } else {
                    Err("Invalid topic")
                }
            }
        }
    }
}

impl From<MqttTopic> for String<100> {
    fn from(v: MqttTopic) -> Self {
        let mut topic = String::<100>::new();
        match v {
            MqttTopic::State => topic.push_str("hyped/poddington/state/state").unwrap(),
            MqttTopic::StateRequest => topic
                .push_str("hyped/poddington/state/state_request")
                .unwrap(),
            MqttTopic::Heartbeat => topic.push_str("hyped/poddington/heartbeat").unwrap(),
            MqttTopic::Logs => topic.push_str("hyped/poddington/logs").unwrap(),
            MqttTopic::LatencyRequest => topic.push_str("hyped/poddington/latency/request").unwrap(),
            MqttTopic::LatencyResponse => topic.push_str("hyped/poddington/latency/response").unwrap(),
            MqttTopic::Debug => topic.push_str("debug").unwrap(),
            MqttTopic::Test => topic.push_str("test").unwrap(),
            MqttTopic::Measurement(measurement_id) => {
                topic.push_str(MQTT_MEASUREMENT_TOPIC_PREFIX).unwrap();
                topic.push_str(measurement_id.into()).unwrap();
            }
        }
        topic
    }
}
