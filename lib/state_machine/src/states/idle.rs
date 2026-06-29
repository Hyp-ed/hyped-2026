use crate::{state::State, state_machine::StateMachine};
use defmt::warn;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_idle(&mut self) {
        info!("Pod is idle");
        self.battery_precharge_relay_open = false;
        self.motor_controller_relay_open = false;
        self.motor_controller_setup_command_sent = false;
        self.motor_controller_setup_done = false;
        self.motor_controller_operational_command_sent = false;
        self.motor_controller_operational = false;
        self.queue_publish(Event::OpenPrechargeRelaysCommand);
        // Send clamp brakes command in case brakes aren't clamped yet
        self.queue_publish(Event::ClampBrakesCommand);
        self.brakes_clamped = false;
    }

    pub(crate) async fn react_idle(&mut self, event: Event) {
        match event {
            // TODOLater: Add unclamp brakes operator command?
            Event::BrakesClamped { from } => {
                info!("Brakes clamped (from:{}, can now begin precharge", from);
                self.brakes_clamped = true;
            }
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
            }
            Event::PrechargeOperatorCommand => {
                info!("Motor setup command received");
                if self.brakes_clamped {
                    self.transition_to(State::SetupMotor).await;
                } else {
                    warn!("Brakes are not clamped, cannot begin motor setup.")
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
