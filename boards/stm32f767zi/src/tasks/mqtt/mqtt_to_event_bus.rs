use super::receive::MQTT_RECEIVE;
use hyped_communications::{bus::EVENT_BUS, events::Event};
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
            "start" => Some(Event::StartRunOperatorCommand),
            "stop" => Some(Event::BrakeOperatorCommand),
            // TODOLater: check if emergency stop needed
            _ => None,
        };
        if let Some(event) = event {
            EVENT_BUS.sender().send(event).await;
        }
    }
}
