use core::str::FromStr;
use embassy_futures::join::join;
use heapless::String;
use hyped_core::{
    format, format_string::show, mqtt::MqttMessage, mqtt_topics::MQTT_MEASUREMENT_TOPIC_PREFIX,
};

use super::{can::receive::INCOMING_MEASUREMENTS, mqtt::send::MQTT_SEND};

/// Run functions to send CAN messages to MQTT and vice versa.
#[embassy_executor::task]
pub async fn can_to_mqtt() {
    join(
        join(
            send_can_measurement_to_mqtt(),
            async {}, //TODO  Placeholder
        ),
        async {}, //TODO Placeholder
    )
    .await;
}

/// Send a CAN measurement to MQTT.
pub async fn send_can_measurement_to_mqtt() {
    let measurements_receiver = INCOMING_MEASUREMENTS.receiver();

    defmt::debug!("Task started: send_can_measurement_to_mqtt");

    loop {
        let measurement = measurements_receiver.receive().await;

        let mut topic_string = String::<100>::new();
        topic_string
            .push_str(MQTT_MEASUREMENT_TOPIC_PREFIX)
            .unwrap();
        topic_string
            .push_str(measurement.measurement_id.into())
            .unwrap();

        let topic = topic_string
            .parse()
            .expect("Failed to parse measurement ID from CAN bus");

        let payload =
            String::from_str(format!(&mut [0u8; 1024], "{}", measurement.reading).unwrap())
                .unwrap();

        let message = MqttMessage::new(topic, payload);

        defmt::debug!("Sending CAN measurement to MQTT: {:?}", message);

        MQTT_SEND.send(message).await;
    }
}
