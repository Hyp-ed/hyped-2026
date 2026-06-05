use super::receive::MQTT_RECEIVE;
use hyped_communications::{boards::Board, bus::EVENT_BUS, events::Event};
use hyped_core::mqtt_topics::MqttTopic;

#[embassy_executor::task]
pub async fn mqtt_to_event_bus() {
    let receiver = MQTT_RECEIVE.receiver();
    loop {
        let message = receiver.receive().await;
        if message.topic != MqttTopic::Controls {
            continue;
        }
        defmt::info!("Received MQTT message on topic {:?} with payload {:?}", message.topic, message.payload);
        let event = match message.payload.as_str() {
            "start-hp" => Some(Event::PrechargeOperatorCommand),
            "start-run" => Some(Event::StartRunOperatorCommand),
            "accelerate" => Some(Event::AccelerateOperatorCommand),
            "stop" => Some(Event::BrakeOperatorCommand),
            "emergency-stop" => Some(Event::EmergencyStopOperatorCommand),
            //TESTING PURPOSES: for single-board testing, just map 'clamp' and 'retract' to response --> will be removed
            "clamp" => Some(Event::BrakesClamped { from: Board::Telemetry }),
            "retract" => Some(Event::LateralSuspensionRetracted { from: Board::Telemetry }),
            _ => None,
        };
        if let Some(event) = event {
            EVENT_BUS.sender().send(event).await;
        }
    }
}
