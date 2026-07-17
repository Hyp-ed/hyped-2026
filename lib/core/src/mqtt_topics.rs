// TODOLater: pod name

use crate::config::MeasurementId;
use core::str::FromStr;
use heapless::String;

pub const MQTT_MEASUREMENT_TOPIC_PREFIX: &str = "hyped/the_podigal_son/measurement/";
pub const MQTT_CONTROLS_TOPIC_PREFIX: &str = "hyped/the_podigal_son/controls/";

/// Enum representing all MQTT topics used by the pod
#[derive(Debug, defmt::Format, PartialEq, Eq)]
pub enum MqttTopic {
    Measurement(MeasurementId),
    State,
    ControlStatus,
    ImdStatus,
    HvalRedStatus,
    HvalGreenStatus,
    BrakeClampStatus,
    Controls,
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
            "hyped/the_podigal_son/state" | "hyped/the_podigal_son/state/state" => {
                Ok(MqttTopic::State)
            }
            "hyped/the_podigal_son/control-status" => Ok(MqttTopic::ControlStatus),
            "hyped/the_podigal_son/status/imd_status" => Ok(MqttTopic::ImdStatus),
            "hyped/the_podigal_son/status/hval_red_status" => Ok(MqttTopic::HvalRedStatus),
            "hyped/the_podigal_son/status/hval_green_status" => Ok(MqttTopic::HvalGreenStatus),
            "hyped/the_podigal_son/status/brake_clamp_status" => Ok(MqttTopic::BrakeClampStatus),
            "hyped/the_podigal_son/heartbeat" => Ok(MqttTopic::Heartbeat),
            "hyped/the_podigal_son/logs" => Ok(MqttTopic::Logs),
            "hyped/the_podigal_son/latency/request" => Ok(MqttTopic::LatencyRequest),
            "hyped/the_podigal_son/latency/response" => Ok(MqttTopic::LatencyResponse),
            "hyped/the_podigal_son/controls" => Ok(MqttTopic::Controls),
            "debug" => Ok(MqttTopic::Debug),
            "test" => Ok(MqttTopic::Test),
            _ => {
                if s.starts_with(MQTT_MEASUREMENT_TOPIC_PREFIX) {
                    let measurement_id_string = &s[MQTT_MEASUREMENT_TOPIC_PREFIX.len()..s.len()];
                    let measurement_id = measurement_id_string.into();
                    Ok(MqttTopic::Measurement(measurement_id))
                } else if s.starts_with(MQTT_CONTROLS_TOPIC_PREFIX) {
                    Ok(MqttTopic::Controls)
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
            MqttTopic::State => topic.push_str("hyped/the_podigal_son/state").unwrap(),
            MqttTopic::ControlStatus => topic
                .push_str("hyped/the_podigal_son/control-status")
                .unwrap(),
            MqttTopic::ImdStatus => topic
                .push_str("hyped/the_podigal_son/status/imd_status")
                .unwrap(),
            MqttTopic::HvalRedStatus => topic
                .push_str("hyped/the_podigal_son/status/hval_red_status")
                .unwrap(),
            MqttTopic::HvalGreenStatus => topic
                .push_str("hyped/the_podigal_son/status/hval_green_status")
                .unwrap(),
            MqttTopic::BrakeClampStatus => topic
                .push_str("hyped/the_podigal_son/status/brake_clamp_status")
                .unwrap(),
            MqttTopic::Controls => topic.push_str("hyped/the_podigal_son/controls").unwrap(),
            MqttTopic::Heartbeat => topic.push_str("hyped/the_podigal_son/heartbeat").unwrap(),
            MqttTopic::Logs => topic.push_str("hyped/the_podigal_son/logs").unwrap(),
            MqttTopic::LatencyRequest => topic
                .push_str("hyped/the_podigal_son/latency/request")
                .unwrap(),
            MqttTopic::LatencyResponse => topic
                .push_str("hyped/the_podigal_son/latency/response")
                .unwrap(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_status_topics_round_trip() {
        for (topic, expected) in [
            (
                MqttTopic::ImdStatus,
                "hyped/the_podigal_son/status/imd_status",
            ),
            (
                MqttTopic::HvalRedStatus,
                "hyped/the_podigal_son/status/hval_red_status",
            ),
            (
                MqttTopic::HvalGreenStatus,
                "hyped/the_podigal_son/status/hval_green_status",
            ),
            (
                MqttTopic::BrakeClampStatus,
                "hyped/the_podigal_son/status/brake_clamp_status",
            ),
        ] {
            let encoded: String<100> = topic.into();
            assert_eq!(encoded.as_str(), expected);
            let decoded: MqttTopic = encoded.parse().expect("status topic should parse");
            let reencoded: String<100> = decoded.into();
            assert_eq!(reencoded.as_str(), expected);
        }
    }
}
