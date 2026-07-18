use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_entering_maintenance(&mut self) {
        info!("Preparing Maintenance: isolating HV and retracting brakes");
        self.shutdown_circuitry_relay_open = false;
        self.battery_precharge_relay_open = false;
        self.motor_controller_relay_open = false;
        self.brakes_clamped = true;
        self.motor_controller_operational = false;

        self.queue_publish(Event::StartPropulsionBrakingCommand);
        self.queue_publish(Event::OpenPrechargeRelaysCommand);
        self.queue_publish(Event::UnclampBrakesCommand);
    }

    pub(crate) async fn react_entering_maintenance(&mut self, event: Event) {
        match event {
            Event::ShutdownCircuitryRelayOpen => {
                self.shutdown_circuitry_relay_open = true;
                self.complete_maintenance_entry_if_safe().await;
            }
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
                self.complete_maintenance_entry_if_safe().await;
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
                self.complete_maintenance_entry_if_safe().await;
            }
            Event::BrakesUnclamped { from } => {
                info!("Maintenance brakes retracted by {}", from);
                self.brakes_clamped = false;
                self.complete_maintenance_entry_if_safe().await;
            }
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayClosed
            | Event::MotorControllerOperational
            | Event::PropulsionAccelerationStarted => {
                warn!("Unsafe activity while preparing Maintenance");
                self.transition_to(State::Emergency).await;
            }
            Event::BrakesClamped { .. } => {
                warn!("Brakes failed to remain retracted while preparing Maintenance");
                self.transition_to(State::Emergency).await;
            }
            _ => debug!("Event {} is ignored while preparing Maintenance", event),
        }
    }

    pub(crate) async fn entry_maintenance(&mut self) {
        info!("Maintenance condition confirmed");
    }

    pub(crate) async fn react_maintenance(&mut self, event: Event) {
        match event {
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayClosed
            | Event::DischargeRelayClosed
            | Event::MotorControllerOperational
            | Event::PropulsionAccelerationStarted
            | Event::BrakesClamped { .. } => {
                warn!("Maintenance invariant violated");
                self.transition_to(State::Emergency).await;
            }
            _ => debug!("Event {} is ignored in Maintenance", event),
        }
    }

    async fn complete_maintenance_entry_if_safe(&mut self) {
        if self.shutdown_circuitry_relay_open
            && self.battery_precharge_relay_open
            && self.motor_controller_relay_open
            && !self.brakes_clamped
        {
            self.transition_to(State::Maintenance).await;
        }
    }
}
