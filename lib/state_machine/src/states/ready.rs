use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;

use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_ready(&mut self) {
        info!("Pod is ready");
        info!("Awaiting accelerate command from operator")
    }

    pub(crate) async fn react_ready(&mut self, event: Event) {
        match event {
            // GO
            Event::AccelerateOperatorCommand => {
                // TODO: do we need to have been ready for a certain threshold of time?
                info!("Starting acceleration");
                self.transition_to(State::Accelerate).await;
            }
            // Status
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
            // Abort
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            Event::StopLevitationCommand => {
                info!("Stop levitation pressed");
                self.transition_to(State::StopLevitation).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
