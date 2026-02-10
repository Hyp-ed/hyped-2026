use crate::tasks::can::send::CAN_SEND;
use hyped_communications::{bus::EVENT_BUS, events::Event, messages::CanMessage};

// Bridges event_bus to CAN bus.
// Listens for events and converts them to Can massages
#[embassy_executor::task]
pub async fn event_to_can() -> ! {
    let can_sender = CAN_SEND.sender();
    let event_receiver = EVENT_BUS.receiver();

    loop {
        let event = event_receiver.receive().await;

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

            // Electronics
            Event::StartPrechargeCommand => Some(CanMessage::StartPrechargeCommand),
            Event::StartDischargeCommand => Some(CanMessage::StartDischargeCommand),
            Event::PrechargeStarted => Some(CanMessage::PrechargeStarted),
            Event::DischargeStarted => Some(CanMessage::DischargeStarted),
            Event::PrechargeComplete => Some(CanMessage::PrechargeComplete),
            Event::DischargeComplete => Some(CanMessage::DischargeComplete),
            Event::VoltageStatus { voltage } => Some(CanMessage::VoltageStatus { voltage }),
            Event::PrechargeVoltageOK => Some(CanMessage::PrechargeVoltageOK),
            Event::DischargeVoltageOK => Some(CanMessage::DischargeVoltageOK),

            // Relays
            Event::ShutdownCircuitryRelayOpen => Some(CanMessage::ShutdownCircuitryRelayOpen),
            Event::ShutdownCircuitryRelayClosed => Some(CanMessage::ShutdownCircuitryRelayClosed),
            Event::BatteryPrechargeRelayOpen => Some(CanMessage::BatteryPrechargeRelayOpen),
            Event::BatteryPrechargeRelayClosed => Some(CanMessage::BatteryPrechargeRelayClosed),
            Event::MotorControllerRelayOpen => Some(CanMessage::MotorControllerRelayOpen),
            Event::MotorControllerRelayClosed => Some(CanMessage::MotorControllerRelayClosed),
            Event::DischargeRelayOpen => Some(CanMessage::DischargeRelayOpen),
            Event::DischargeRelayClosed => Some(CanMessage::DischargeRelayClosed),

            // Navigation
            Event::EndOfTrackBrakeCommand => Some(CanMessage::EndOfTrackBrake),

            // Dynamics
            Event::UnclampBrakesCommand => Some(CanMessage::UnclampBrakesCommand),
            Event::ClampBrakesCommand => Some(CanMessage::ClampBrakesCommand),
            Event::RetractLateralSuspensionCommand => {
                Some(CanMessage::RetractLateralSuspensionCommand)
            }
            Event::ExtendLateralSuspensionCommand => {
                Some(CanMessage::ExtendLateralSuspensionCommand)
            }

            Event::BrakesClamped { from } => Some(CanMessage::BrakesClamped { from }),
            Event::BrakesUnclamped { from } => Some(CanMessage::BrakesUnclamped { from }),
            Event::LateralSuspensionRetracted { from } => {
                Some(CanMessage::LateralSuspensionRetracted { from })
            }
            Event::LateralSuspensionExtended { from } => {
                Some(CanMessage::LateralSuspensionExtended { from })
            }
            Event::DynamicsStatus {
                from,
                actuator_pressure_bar,
            } => Some(CanMessage::DynamicsStatus {
                from,
                actuator_pressure_bar,
            }),

            //   Propulsion
            Event::StartPropulsionAccelerationCommand => {
                Some(CanMessage::StartPropulsionAccelerationCommand)
            }
            Event::StartPropulsionBrakingCommand => Some(CanMessage::StartPropulsionBrakingCommand),
            Event::PropulsionAccelerationStarted => Some(CanMessage::PropulsionAccelerationStarted),
            Event::PropulsionBrakingStarted => Some(CanMessage::PropulsionBrakingStarted),
            Event::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => Some(CanMessage::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            }),
            Event::PropulsionForce { force_n } => Some(CanMessage::PropulsionForce { force_n }),
        };

        if let Some(msg) = can_message {
            can_sender.send(msg).await;
        }
    }
}
