use crate::{state_enum::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info};

impl StateMachine {
    pub(crate) async fn entry_idle(&mut self) {
        info!("Pod is idle");
    }

    pub(crate) async fn react_idle(&mut self, event: Event) {
        match event {
            Event::CalibrateOperatorCommand => {
                info!("Calibrate command received");
                self.transition_to(State::Calibrate).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
