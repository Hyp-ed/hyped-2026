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
        self.precharge_complete = false;
        self.ready_for_run = false;
        self.battery_precharge_relay_open = false;
        self.motor_controller_relay_open = false;
        self.motor_controller_operational_command_sent = false;
        self.motor_controller_operational = false;
        self.queue_publish(Event::StartPrechargeCommand);
    }

    pub(crate) async fn react_precharge(&mut self, event: Event) {
        match event {
            Event::PrechargeStarted => {
                info!("Started precharge at {}ms", Instant::now().as_millis(),);
            }
            Event::PrechargeComplete => {
                info!("Completed precharge at {}ms", Instant::now().as_millis(),);
                self.precharge_complete = true;
                self.mark_ready_for_run_if_precharge_complete();
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
                    self.battery_precharge_relay_open = false;
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }
            Event::MotorControllerRelayClosed => {
                if self.precharge_step == PrechargeStep::BatteryPrechargeClosed {
                    self.precharge_step = PrechargeStep::AllClosed;
                    self.motor_controller_relay_open = false;
                    info!("Necessary relays for precharge closed");
                } else {
                    warn!("Relays are out of order!");
                    self.transition_to(State::Emergency).await;
                }
            }

            // todolater: consider adding in an exit (that's not emergency) if motor controller setup is not true

            // Any other change in relays, goto Emergency
            Event::DischargeRelayClosed
            | Event::ShutdownCircuitryRelayOpen
            | Event::BatteryPrechargeRelayOpen
            | Event::MotorControllerRelayOpen => {
                warn!("Unexpected relay change during precharge");
                self.transition_to(State::Emergency).await;
            }

            Event::PrechargeVoltageOK => {
                self.precharge_voltage_ok = true;
                self.mark_ready_for_run_if_precharge_complete();
            }
            Event::ReadyForPropulsionOperatorCommand => {
                if self.ready_for_run {
                    info!("Operator confirmed pod is ready for propulsion");
                    self.transition_to(State::ReadyForPropulsion).await;
                } else {
                    warn!("Precharge incomplete, cannot enter ready for propulsion");
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

    fn mark_ready_for_run_if_precharge_complete(&mut self) {
        if self.precharge_voltage_ok
            && self.precharge_complete
            && self.precharge_step == PrechargeStep::AllClosed
        {
            self.ready_for_run = true;
            info!("Precharge complete; awaiting ready for propulsion command");
        } else if self.precharge_complete && !self.precharge_voltage_ok {
            warn!("Precharge voltage not at accepted value");
        }
    }
}
