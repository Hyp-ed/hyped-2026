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
            // Operator Commands
            Event::EmergencyStopOperatorCommand => None,
            Event::CalibrateOperatorCommand => None,
            Event::BeginLevitationOperatorCommand => None,
            Event::AccelerateOperatorCommand => None,
            Event::BrakeOperatorCommand => None,
            Event::StopLevitationOperatorCommand => None,

            // Emergency
            Event::Emergency { from, reason } => Some(CanMessage::Emergency(from, reason)),

            // Heartbeat (handled by separate heartbeat task)
            Event::Heartbeat { .. } => None,

            // Calibration
            Event::StartCalibrationCommand => Some(CanMessage::StartCalibrationCommand),
            Event::CalibrationComplete { from } => Some(CanMessage::CalibrationComplete { from }),

            // Electronics
            Event::StartPrechargeCommand => Some(CanMessage::StartPrechargeCommand),
            Event::StartDischargeCommand => Some(CanMessage::StartDischargeCommand),
            Event::PrechargeStarted { from } => Some(CanMessage::PrechargeStarted { from }),
            Event::DischargeStarted { from } => Some(CanMessage::DischargeStarted { from }),
            Event::PrechargeComplete { from, voltage_cv } => Some(CanMessage::PrechargeComplete {
                from,
                voltage: voltage_cv,
            }),
            Event::DischargeComplete { from, voltage_cv } => Some(CanMessage::DischargeComplete {
                from,
                voltage: voltage_cv,
            }),
            Event::PrechargeFailed { from, reason } => {
                Some(CanMessage::PrechargeFailed { from, reason })
            }

            // Levitation
            Event::LevitationSystemsReady => Some(CanMessage::LevitationSystemsReady),
            Event::StartLevitationCommand => Some(CanMessage::StartLevitationCommand),
            Event::StopLevitationCommand => Some(CanMessage::StopLevitationCommand),
            Event::LevitationStarted { from } => Some(CanMessage::LevitationStarted { from }),
            Event::LevitationStatus {
                from,
                airgap_μm,
                current_ma,
            } => Some(CanMessage::LevitationStatus {
                from,
                current_ma,
                airgap_μm,
            }),
            Event::LevitationStopped { from } => Some(CanMessage::LevitationStopped { from }),
            Event::LevitationFailed { from, reason } => {
                Some(CanMessage::LevitationFailed { from, reason })
            }

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
            Event::PropulsionFailed { from, reason } => {
                Some(CanMessage::PropulsionFailed { from, reason })
            }
        };

        if let Some(msg) = can_message {
            can_sender.send(msg).await;
        }
    }
}
