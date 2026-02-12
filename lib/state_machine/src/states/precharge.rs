use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_precharge(&mut self) {
        info!("Starting precharge");
        self.precharge_step = 0;
        self.precharge_voltage_ok = false;
        self.ready_for_run = false;
        EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
    }

    pub(crate) async fn react_precharge(&mut self, event: Event) {
        match event {
            Event::PrechargeStarted => {
                info!("Started precharge at {}ms", Instant::now().as_millis(),);
            }
            Event::PrechargeComplete => {
                info!("Completed precharge at {}ms", Instant::now().as_millis(),);
                if self.precharge_voltage_ok && self.precharge_step == 3 {
                    self.ready_for_run = true;
                    info!("Awaiting start run command from operator")
                } else {
                    warn!("Precharge voltage not at accepted value")
                }
            }
            Event::VoltageStatus { voltage } => {
                info!("Voltage {} at {}ms", voltage, Instant::now().as_millis(),);
            }
            Event::ShutdownCircuitryRelayClosed => {
                if self.precharge_step == 0 {
                    self.precharge_step = 1;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::BatteryPrechargeRelayClosed => {
                if self.precharge_step == 1 {
                    self.precharge_step = 2;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::MotorControllerRelayClosed => {
                if self.precharge_step == 2 {
                    self.precharge_step = 3;
                    info!("Necessary relays for precharge closed");
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }

            // Any other change in relays, goto Emergency
            Event::DischargeRelayClosed
            | Event::ShutdownCircuitryRelayOpen
            | Event::BatteryPrechargeRelayOpen
            | Event::MotorControllerRelayOpen => {
                warn!("Unexpected relay change during precharge");
                self.transition_to(State::Emergency).await;
            }

            Event::StartRunOperatorCommand => {
                if self.ready_for_run {
                    info!("Starting Propulsion run");
                    self.transition_to(State::ReadyForPropulsion).await;
                }
            }
            Event::PrechargeVoltageOK => {
                self.precharge_voltage_ok = true;

                // In case this event arrives after PrechargeComplete
                if self.precharge_step == 3 {
                    self.ready_for_run = true;
                }
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
}
