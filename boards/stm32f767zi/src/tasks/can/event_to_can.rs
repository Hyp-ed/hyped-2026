use crate::tasks::can::send::CAN_SEND;
use hyped_communications::{bus::DynSubscriber, events::Event, messages::CanMessage};

// Bridges event_bus to CAN bus.
// Listens for events and converts them to Can massages
#[embassy_executor::task]
pub async fn event_to_can(mut events: DynSubscriber<'static, Event>) -> ! {
    let can_sender = CAN_SEND.sender();

    loop {
        let event = events.next_message_pure().await;

        let can_message: Option<CanMessage> = match event {
            // Operator Commands (not sent over CAN)
            Event::EmergencyStopOperatorCommand => None,
            Event::PrechargeOperatorCommand => None,
            Event::AccelerateOperatorCommand => None,
            Event::BrakeOperatorCommand => None,
            Event::StartRunOperatorCommand => None,

            // Emergency
            Event::Emergency { from, reason } => Some(CanMessage::Emergency(from, reason)),

            // Heartbeat (handled by separate heartbeat task)
            Event::Heartbeat { .. } => None,

            // Outbound commands from the state machine
            Event::StartPrechargeCommand => Some(CanMessage::StartPrechargeCommand),
            Event::StartDischargeCommand => Some(CanMessage::StartDischargeCommand),
            Event::EndOfTrackBrakeCommand => Some(CanMessage::EndOfTrackBrake),
            Event::UnclampBrakesCommand => Some(CanMessage::UnclampBrakesCommand),
            Event::ClampBrakesCommand => Some(CanMessage::ClampBrakesCommand),
            Event::RetractLateralSuspensionCommand => {
                Some(CanMessage::RetractLateralSuspensionCommand)
            }
            Event::ExtendLateralSuspensionCommand => {
                Some(CanMessage::ExtendLateralSuspensionCommand)
            }
            Event::StartPropulsionAccelerationCommand => {
                Some(CanMessage::StartPropulsionAccelerationCommand)
            }
            Event::StartPropulsionBrakingCommand => Some(CanMessage::StartPropulsionBrakingCommand),

            // Ingress-only: status and completion events from other boards
            Event::PrechargeStarted
            | Event::DischargeStarted
            | Event::PrechargeComplete
            | Event::DischargeComplete
            | Event::VoltageStatus { .. }
            | Event::PrechargeVoltageOK
            | Event::DischargeVoltageOK
            | Event::ShutdownCircuitryRelayOpen
            | Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayOpen
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayOpen
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayOpen
            | Event::DischargeRelayClosed
            | Event::BrakesClamped { .. }
            | Event::BrakesUnclamped { .. }
            | Event::LateralSuspensionRetracted { .. }
            | Event::LateralSuspensionExtended { .. }
            | Event::DynamicsStatus { .. }
            | Event::PropulsionAccelerationStarted
            | Event::PropulsionBrakingStarted
            | Event::PropulsionStatus { .. }
            | Event::PropulsionForce { .. } => None,
        };

        if let Some(msg) = can_message {
            can_sender.send(msg).await;
        }
    }
}
