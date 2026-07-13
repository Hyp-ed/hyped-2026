use crate::{
    state::State,
    state_machine::{PrechargeStep, StateMachine},
};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    /// Regulatory HV Active condition: SDC closed and no propulsion request is issued.
    pub(crate) async fn entry_hv_active(&mut self) {
        info!("HV active; propulsion remains disabled until Demo is requested");
        self.motor_controller_operational = false;
        self.queue_publish(Event::StartPropulsionBrakingCommand);
    }

    pub(crate) async fn react_hv_active(&mut self, event: Event) {
        match event {
            Event::AccelerateOperatorCommand => {
                if self.precharge_step != PrechargeStep::AllClosed {
                    warn!("Cannot begin Demo: SDC/precharge sequence is not complete");
                    self.transition_to(State::Emergency).await;
                } else {
                    self.transition_to(State::ReadyForPropulsion).await;
                }
            }
            Event::ShutdownCircuitryRelayOpen
            | Event::BatteryPrechargeRelayOpen
            | Event::MotorControllerRelayOpen
            | Event::BrakesUnclamped { .. }
            | Event::PropulsionAccelerationStarted => {
                warn!("HV Active invariant violated");
                self.transition_to(State::Emergency).await;
            }
            _ => debug!("Event {} is ignored in HV Active", event),
        }
    }
}
