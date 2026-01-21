use crate::{state::State, state_machine::StateMachine};
use hyped_communications::events::Event;
use hyped_core::types::Airgap;

use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    // --------- READY ---------

    pub(crate) async fn entry_ready(&mut self) {
        info!("Pod is ready");
        info!("Levitation is stable, awaiting accelerate command from operator")
    }

    pub(crate) async fn react_ready(&mut self, event: Event) {
        match event {
            // GO
            Event::AccelerateOperatorCommand => {
                // TODO do we need to have been ready for a certain threshold of time?
                info!("Starting acceleration");
                self.transition_to(State::Accelerate).await;
            }
            Event::LevitationStatus {
                from,
                airgap_μm,
                current_ma,
            } => {
                info!("board={}, current={}", from, current_ma);
                let target_airgap_μm = Airgap(5000); // TODO confirm a figure for this
                let dist_to_target = airgap_μm.distance_to(target_airgap_μm);

                // Handle if we drift too far from target
                if dist_to_target > 7000 {
                    // TODO check with levitation team how big
                    warn!("Levitation unstable: {}μm from target", dist_to_target);
                }
            }
            // TODO decide if we need this
            Event::LevitationFailed { from, reason } => {
                warn!("Levitation failed: reason={}, board={}", reason, from);
                self.transition_to(State::Emergency).await;
            }

            // Abort
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            Event::StopLevitationCommand => {
                info!("Stop levitation requested");
                self.transition_to(State::StopLevitation).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
