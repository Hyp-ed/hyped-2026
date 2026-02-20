use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_stopped(&mut self) {
        info!("Pod is stopped");
        EVENT_BUS.sender().send(Event::StartDischargeCommand).await;
        self.discharge_step = 0;
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
                if self.discharge_step == 2 && self.discharge_voltage_ok {
                    info!("Discharge completed successfully ");
                    self.transition_to(State::Idle).await;
                } else {
                    warn!("Discharge did not complete successfully")
                }
            }
            Event::DischargeRelayClosed => {
                if self.discharge_step == 0 {
                    self.discharge_step = 1;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::ShutdownCircuitryRelayOpen => {
                if self.discharge_step == 1 {
                    self.discharge_step = 2;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }

            // Any other change in relays, goto Emergency
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayOpen
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayOpen
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayOpen => {
                warn!("Unexpected relay change during discharge");
                self.transition_to(State::Emergency).await;
            }

            Event::DischargeVoltageOK => {
                self.discharge_voltage_ok = true;

                // In case this event arrives after DischargeComplete
                if self.discharge_step == 2 {
                    self.transition_to(State::Idle).await;
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
