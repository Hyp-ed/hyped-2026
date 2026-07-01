use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_stopped(&mut self) {
        info!("Pod is stopped");
        self.queue_publish(Event::StartDischargeCommand);
        self.discharge_voltage_ok = false;
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
                if self.discharge_voltage_ok {
                    info!("Discharge completed successfully ");
                    self.transition_to(State::Idle).await;
                } else {
                    warn!("Discharge did not complete successfully")
                }
            }
            Event::ShutdownCircuitryRelayOpen => {
                info!("Shutdown circuitry relay opened for discharge");
            }

            // Any other change in relays, goto Emergency
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayOpen
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayOpen
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayClosed
            | Event::DischargeRelayOpen => {
                warn!("Unexpected relay change during discharge");
                self.transition_to(State::Emergency).await;
            }

            Event::DischargeVoltageOK => {
                self.discharge_voltage_ok = true;

                // In case this event arrives after DischargeComplete
                if self.discharge_voltage_ok {
                    self.transition_to(State::Idle).await;
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
