use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_levitating(&mut self) {
        info!("Pod is levitating");
    }

    pub(crate) async fn react_levitating(&mut self, event: Event) {
        match event {
            Event::LevitationStatus {
                from,
                airgap_μm,
                current_ma,
            } => {
                info!(
                    "board={}, current={}, airgap={}",
                    from, current_ma, airgap_μm
                );
            }
            // Stop
            Event::StopLevitationOperatorCommand => {
                info!("Stop levitation pressed");
                self.transition_to(State::StopLevitation).await;
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
