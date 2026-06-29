use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_setup_motor(&mut self) {
        info!("Setting up motor controller");
        self.motor_controller_setup_command_sent = false;
        self.motor_controller_setup_done = false;
        self.motor_controller_operational_command_sent = false;
        self.motor_controller_operational = false;
        self.command_motor_controller_setup_if_precharge_relays_open();
    }

    pub(crate) async fn react_setup_motor(&mut self, event: Event) {
        match event {
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
                self.command_motor_controller_setup_if_precharge_relays_open();
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
                self.command_motor_controller_setup_if_precharge_relays_open();
            }
            Event::MotorControllerSetupComplete => {
                if self.battery_precharge_relay_open && self.motor_controller_relay_open {
                    self.motor_controller_setup_done = true;
                    info!("Motor controller setup complete; awaiting precharge command");
                } else {
                    warn!("Motor controller setup completed before precharge relays were open");
                }
            }
            Event::StartRunOperatorCommand => {
                if self.motor_controller_setup_done {
                    info!("Entering precharge");
                    self.transition_to(State::Precharge).await;
                } else {
                    warn!("Motor controller setup incomplete, cannot enter precharge");
                }
            }
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayClosed => {
                warn!("Unexpected relay closure during motor setup");
                self.transition_to(State::Emergency).await;
            }
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }

    fn command_motor_controller_setup_if_precharge_relays_open(&mut self) {
        if self.battery_precharge_relay_open && self.motor_controller_relay_open {
            if self.motor_controller_setup_command_sent {
                return;
            }
            self.motor_controller_setup_command_sent = true;
            self.queue_publish(Event::MotorControllerSetupCommand);
        }
    }
}
