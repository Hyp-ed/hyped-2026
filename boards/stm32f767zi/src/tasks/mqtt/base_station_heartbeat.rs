use defmt::debug;
use defmt_rtt as _;
use embassy_time::Duration;
use hyped_core::{config::HEARTBEAT_CONFIG, mqtt_topics::MqttTopic};
use panic_probe as _;
use crate::tasks::mqtt::receive::MQTT_HEARTBEAT_RECEIVE;
use hyped_communications::{events::Reason, messages::CanMessage};
use crate::emergency;
use crate::tasks::can::send::CAN_SEND;
use crate::board_state::{EMERGENCY, THIS_BOARD};

/// Listens for heartbeat messages from the MQTT broker and triggers emergency brakes if no heartbeat is received within the expected time frame.
#[embassy_executor::task]
pub async fn base_station_heartbeat_listener() {
    match wait_for_first_base_station_heartbeat().await {
        Ok(_) => {
            debug!("Base station is alive!");
        }
        Err(_) => {
            defmt::error!(
                "Failed to receive first heartbeat from base station"
            );
            emergency!(Reason::NoInitialBaseStationHeartbeat);
        }
    }

    loop {
        let result = embassy_time::with_timeout(
            Duration::from_millis(HEARTBEAT_CONFIG.base_station.max_latency_ms as u64),
            async {
                loop {
                    let message = MQTT_HEARTBEAT_RECEIVE.receive().await;
                    if message.topic == MqttTopic::Heartbeat {
                        break;
                    }
                }
            },
        )
        .await;

        match result {
            Ok(_) => {
                debug!("Received heartbeat message");
            }
            Err(_) => {
                defmt::error!(
                    "Emergency stop triggered due to missing heartbeat from base station"
                );
                emergency!(Reason::MissingBaseStationHeartbeat);
            }
        }
    }
}

/// Waits for the first heartbeat message from the base station and returns an error if it is not received within the expected time frame.
pub async fn wait_for_first_base_station_heartbeat() -> Result<(), ()> {
    let result = embassy_time::with_timeout(
        Duration::from_secs(HEARTBEAT_CONFIG.base_station.startup_timeout_s as u64),
        async {
            loop {
                let message = MQTT_HEARTBEAT_RECEIVE.receive().await;
                if message.topic == MqttTopic::Heartbeat {
                    break;
                }
            }
        },
    )
    .await;

    match result {
        Ok(_) => {
            debug!("Received first heartbeat message from base station");
            Ok(())
        }
        Err(_) => {
            defmt::error!(
                "Failed to receive first heartbeat from base station"
            );
            Err(())
        }
    }
}