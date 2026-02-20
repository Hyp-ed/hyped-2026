use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info};

impl StateMachine {
    pub(crate) async fn entry_idle(&mut self) {
        info!("Pod is idle");
    }

    pub(crate) async fn react_idle(&mut self, event: Event) {
        match event {
            // TODOLater: Add unclamp brakes operator command?
            Event::PrechargeOperatorCommand => {
                info!("Precharge command received");
                self.transition_to(State::Precharge).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
