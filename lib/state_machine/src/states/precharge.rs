use crate::{
    state::State,
    state_machine::{PrechargeStep, StateMachine},
};
use embassy_time::Instant;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_precharge(&mut self) {
        info!("Starting precharge");
        self.precharge_step = PrechargeStep::Initial;
        self.precharge_voltage_ok = false;
        self.ready_for_run = false;
        self.queue_publish(Event::StartPrechargeCommand);
    }

    pub(crate) async fn react_precharge(&mut self, event: Event) {
        match event {
            Event::PrechargeStarted => {
                info!("Started precharge at {}ms", Instant::now().as_millis(),);
            }
            Event::PrechargeComplete => {
                info!("Completed precharge at {}ms", Instant::now().as_millis(),);
                if self.precharge_voltage_ok && self.precharge_step == PrechargeStep::AllClosed {
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
                if self.precharge_step == PrechargeStep::Initial {
                    self.precharge_step = PrechargeStep::ShutdownClosed;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::BatteryPrechargeRelayClosed => {
                if self.precharge_step == PrechargeStep::ShutdownClosed {
                    self.precharge_step = PrechargeStep::BatteryPrechargeClosed;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::MotorControllerRelayClosed => {
                if self.precharge_step == PrechargeStep::BatteryPrechargeClosed {
                    self.precharge_step = PrechargeStep::AllClosed;
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
                if self.precharge_step == PrechargeStep::AllClosed {
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
