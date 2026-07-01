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
            "setup_motor" => Some(Event::PrechargeOperatorCommand),
            "precharge" => Some(Event::StartRunOperatorCommand),
            "ready-for-propulsion" => Some(Event::ReadyForPropulsionOperatorCommand),
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
