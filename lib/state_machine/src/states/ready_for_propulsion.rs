use crate::{state::State, state_machine::StateMachine};
use hyped_communications::{bus::EVENT_BUS, events::Event};

use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_ready_for_propulsion(&mut self) {
        info!("Pod is ready for propulsion");
        info!("Awaiting accelerate command from operator");
        // TODO: are brake clamped or by default?
        EVENT_BUS.sender().send(Event::UnclampBrakesCommand).await;
    }

    pub(crate) async fn react_ready_for_propulsion(&mut self, event: Event) {
        match event {
            // GO
            Event::BrakesUnclamped { from } => {
                info!("Brakes unclamped. board ={}", from);
                self.brakes_clamped = false;
            }
            Event::AccelerateOperatorCommand => {
                // TODO: do we need to have been ready for a certain threshold of time?
                if self.brakes_clamped {
                    warn!("Brakes still clamped, cannot accelerate!");
                } else {
                    info!("Starting acceleration");
                    self.transition_to(State::Accelerate).await;
                }
            }
            // Abort
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
