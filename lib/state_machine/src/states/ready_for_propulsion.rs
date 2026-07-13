use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;

use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_ready_for_propulsion(&mut self) {
        info!("Pod is ready for propulsion");
        info!("Setting motor controller operational");
        self.motor_controller_operational_command_sent = false;
        self.motor_controller_operational = false;
        self.queue_publish(Event::MotorControllerSetOperationalCommand);
        self.motor_controller_operational_command_sent = true;
    }

    pub(crate) async fn react_ready_for_propulsion(&mut self, event: Event) {
        match event {
            Event::MotorControllerOperational => {
                self.motor_controller_operational = true;
                info!("Motor controller operational; unclamping brakes");
                self.queue_publish(Event::UnclampBrakesCommand);
            }
            Event::BrakesUnclamped { from } => {
                info!("Brakes unclamped. board ={}", from);
                self.brakes_clamped = false;
                if self.motor_controller_operational {
                    info!("Demo conditions confirmed; starting acceleration");
                    self.transition_to(State::Accelerate).await;
                } else {
                    warn!("Brakes retracted before the motor controller was operational");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::BatteryPrechargeRelayOpen
            | Event::MotorControllerRelayOpen
            | Event::ShutdownCircuitryRelayOpen => {
                warn!("Unexpected relay opening while ready for propulsion");
                self.transition_to(State::Emergency).await;
            }
            Event::BrakesClamped { .. } | Event::PropulsionAccelerationStarted => {
                warn!("Demo preparation invariant violated");
                self.transition_to(State::Emergency).await;
            }
            // Abort
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
