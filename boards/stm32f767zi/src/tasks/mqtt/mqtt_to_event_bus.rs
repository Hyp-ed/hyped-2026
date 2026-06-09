use super::receive::MQTT_RECEIVE;
use hyped_communications::{bus::publish, events::Event};
use hyped_core::mqtt_topics::MqttTopic;

#[embassy_executor::task]
pub async fn mqtt_to_event_bus() {
    let receiver = MQTT_RECEIVE.receiver();
    loop {
        let message = receiver.receive().await;
        if message.topic != MqttTopic::Controls {
            continue;
        }
        let event = match message.payload.as_str() {
            "start-hp" => Some(Event::PrechargeOperatorCommand),
            "start-run" => Some(Event::StartRunOperatorCommand),
            "accelerate" => Some(Event::AccelerateOperatorCommand),
            "stop" => Some(Event::BrakeOperatorCommand),
            "emergency-stop" => Some(Event::EmergencyStopOperatorCommand),
            _ => None,
        };
        if let Some(event) = event {
            publish(event).await;
        }
    }
}
