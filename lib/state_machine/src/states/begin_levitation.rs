use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    pub(crate) async fn entry_begin_levitation(&mut self) {
        info!("Levitation started");
        EVENT_BUS.sender().send(Event::StartLevitationCommand).await;
    }

    pub(crate) async fn react_begin_levitation(&mut self, event: Event) {
        match event {
            Event::LevitationStarted {
                initial_current_ma,
                initial_airgap_μm,
                target_airgap_μm,
            } => {
                info!(
                    "Status: initial current: {}mA, initial airgap: {}μm, target airgap: {:?}μm",
                    initial_current_ma.0, initial_airgap_μm.0, target_airgap_μm.0
                );
                EVENT_BUS
                    .sender()
                    .send(Event::RetractLateralSuspensionCommand)
                    .await;
            }
            Event::LateralSuspensionRetracted {
                actuator_pressure_bar,
            } => {
                info!(
                    "Lateral suspension retracted: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
                    Instant::now().as_millis(),
                );
            }
            Event::LevitationStatus {
                current_airgap_μm,
                target_airgap_μm,
                current_ma,
            } => {
                // calculate absolute distance
                let dist_to_target = current_airgap_μm.distance_to(target_airgap_μm);
                info!(
                    "Status: current: {:?}mA, distance to target airgap: {}μm",
                    current_ma.0, dist_to_target
                );

                if dist_to_target < 5000 {
                    // TODO later: 5000μm is a placeholder, check with levitation team for real number
                    // TODO later: track how long we've been stable before transitioning to ensure its not a fluctuation
                    info!("Levitation stable, transitioning to Ready");
                    self.transition_to(State::Ready).await;
                }
            }
            Event::LevitationStopped {
                final_airgap_μm,
                final_current_ma,
            } => {} // TODO decide if we need this
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
