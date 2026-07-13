use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_stopped(&mut self) {
        info!("Pod is stopped");
        self.queue_publish(Event::StartDischargeCommand);
        self.discharge_voltage_ok = false;
        self.discharge_complete = false;
    }
    pub(crate) async fn react_stopped(&mut self, event: Event) {
        match event {
            Event::DischargeStarted => {
                // Sent from board
                info!("Started discharge at {}ms", Instant::now().as_millis(),);
            }
            Event::DischargeComplete => {
                // Sent from board
                info!("Completed discharge at {}ms", Instant::now().as_millis(),);
                self.discharge_complete = true;
                self.log_discharge_status();
            }
            Event::ShutdownCircuitryRelayOpen => {
                self.shutdown_circuitry_relay_open = true;
                info!("Shutdown circuitry relay opened for discharge");
            }
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
            }

            // Any other change in relays, goto Emergency
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayClosed => {
                warn!("Unexpected relay change during discharge");
                self.transition_to(State::Emergency).await;
            }
            Event::DischargeRelayOpen => {}

            Event::DischargeVoltageOK => {
                self.discharge_voltage_ok = true;
                self.log_discharge_status();
            }
            Event::IdleOperatorCommand => {
                if self.discharge_voltage_ok && self.discharge_complete {
                    self.transition_to(State::Idle).await;
                } else {
                    warn!("Cannot return to Idle until discharge is confirmed complete");
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }

    fn log_discharge_status(&self) {
        if self.discharge_voltage_ok && self.discharge_complete {
            info!("Discharge complete; Stopped is latched until Idle is requested");
        }
    }
}
