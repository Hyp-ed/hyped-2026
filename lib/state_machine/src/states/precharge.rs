use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_precharge(&mut self) {
        info!("Starting precharge");
        self.precharge_step = 0;
        EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
    }

    pub(crate) async fn react_precharge(&mut self, event: Event) {
        match event {
            Event::PrechargeStarted => {
                info!("Started precharge at {}ms", Instant::now().as_millis(),);
            }
            Event::PrechargeComplete => {
                info!("Completed precharge at {}ms", Instant::now().as_millis(),);
                if self.precharge_voltage_ok {
                    self.ready_for_run = true;
                    info!("Awaiting start run command from operator")
                } else {
                    warn!("Precharge voltage not at accepted value")
                }
            }
            Event::PrechargeVoltageOK => {
                info!("Precharge voltage has reached accepted value");
                self.precharge_voltage_ok = true;
                if self.precharge_step == 3 {
                    self.ready_for_run = true;
                }
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
                    EVENT_BUS.sender().send(Event::PrechargeComplete).await;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::DischargeRelayClosed => {
                warn!("Discharge relay closed unexpectedly");
                self.transition_to(State::Emergency).await;
            }
            Event::ShutdownCircuitryRelayOpen => {
                warn!("SDC relay opened unexpectedly");
                self.transition_to(State::Emergency).await;
            }
            Event::BatteryPrechargeRelayOpen => {
                warn!("Battery Precharge relay opened unexpectedly");
                self.transition_to(State::Emergency).await;
            }
            Event::MotorControllerRelayOpen => {
                warn!("MC relay opened unexpectedly");
                self.transition_to(State::Emergency).await;
            }
            Event::StartRunOperatorCommand => {
                if self.ready_for_run {
                    info!("Starting Propulsion run");
                    self.transition_to(State::ReadyForPropulsion).await;
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
