use crate::{state::State, state_machine::StateMachine};
use defmt::warn;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    pub(crate) async fn entry_idle(&mut self) {
        info!("Pod is idle");
        // Send clamp brakes command in case brakes aren't clamped yet
        EVENT_BUS.sender().send(Event::ClampBrakesCommand).await;
        self.brakes_clamped = false;
    }

    pub(crate) async fn react_idle(&mut self, event: Event) {
        match event {
            // TODOLater: Add unclamp brakes operator command?
            Event::BrakesClamped { from } => {
                info!("Brakes clamped (from:{}, can now begin precharge", from);
                self.brakes_clamped = true;
            }
            Event::PrechargeOperatorCommand => {
                info!("Precharge command received");
                if self.brakes_clamped {
                    self.transition_to(State::Precharge).await;
                } else {
                    warn!("Brakes are not clamped, cannot begin precharge.")
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
