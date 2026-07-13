use core::fmt::Write;

use crate::tasks::{can::send::CAN_SEND, mqtt::send::MQTT_SEND};
use embassy_time::{with_timeout, Duration};
use heapless::String;
use hyped_communications::{bus::DynSubscriber, events::Event, messages::CanMessage};
use hyped_core::{mqtt::MqttMessage, mqtt_topics::MqttTopic};

// Bridges event_bus to CAN bus.
// Listens for events and converts them to Can massages
#[embassy_executor::task]
pub async fn event_to_can(mut events: DynSubscriber<'static, Event>) -> ! {
    let can_sender = CAN_SEND.sender();
    let mqtt_sender = MQTT_SEND.sender();

    loop {
        let event = events.next_message_pure().await;

        let can_message: Option<CanMessage> = match &event {
            // Operator Commands (not sent over CAN)
            Event::EmergencyStopOperatorCommand => None,
            Event::IdleOperatorCommand => None,
            Event::PrechargeOperatorCommand => None,
            Event::AccelerateOperatorCommand => None,
            Event::BrakeOperatorCommand => None,
            Event::StartRunOperatorCommand => None,
            Event::ReadyForPropulsionOperatorCommand => None,

            // Emergency
            Event::Emergency { from, reason } => Some(CanMessage::Emergency(*from, *reason)),

            // Heartbeat (handled by separate heartbeat task)
            Event::Heartbeat { .. } => None,
            Event::StateChanged { state } => {
                let _ = mqtt_sender.try_send(MqttMessage::new_retained_json_string(
                    MqttTopic::State,
                    state,
                ));
                None
            }
            Event::ControlStatusChanged {
                can_setup_motor,
                can_precharge,
                can_ready_for_propulsion,
                can_accelerate,
            } => {
                let mut payload = String::<512>::new();
                let _ = write!(
                    payload,
                    "{{\"canSetupMotor\":{can_setup_motor},\"canPrecharge\":{can_precharge},\"canReadyForPropulsion\":{can_ready_for_propulsion},\"canAccelerate\":{can_accelerate}}}"
                );
                let _ = mqtt_sender
                    .try_send(MqttMessage::new_retained(MqttTopic::ControlStatus, payload));
                None
            }

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
            Event::MotorControllerSetOperationalCommand => {
                Some(CanMessage::MotorControllerSetOperationalCommand)
            }
            Event::MotorControllerSetupCommand => Some(CanMessage::MotorControllerSetupCommand),
            Event::OpenPrechargeRelaysCommand => Some(CanMessage::OpenPrechargeRelaysCommand),

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
            | Event::MotorControllerSetupComplete
            | Event::MotorControllerOperational
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
            let log_command = matches!(
                &msg,
                CanMessage::MotorControllerSetupCommand
                    | CanMessage::MotorControllerSetOperationalCommand
                    | CanMessage::StartPropulsionBrakingCommand
                    | CanMessage::ClampBrakesCommand
                    | CanMessage::UnclampBrakesCommand
            );
            let reliable_brake_command = matches!(
                &msg,
                CanMessage::ClampBrakesCommand | CanMessage::UnclampBrakesCommand
            );

            if log_command {
                defmt::info!("Bridging event to CAN: {:?}", msg);
            }

            if reliable_brake_command {
                can_sender.send(msg).await;
            } else if with_timeout(Duration::from_millis(100), can_sender.send(msg.clone()))
                .await
                .is_err()
            {
                defmt::error!("CAN bridge dropped {:?}: CAN_SEND remained full for 100ms", msg);
                continue;
            }

            if log_command {
                defmt::info!("CAN bridge queued command");
            }
        }

        let mut event_log = String::<512>::new();
        let _ = write!(event_log, "event={event:?}");
        let _ = mqtt_sender.try_send(MqttMessage::new_json_string(
            MqttTopic::Logs,
            event_log.as_str(),
        ));
    }
}
