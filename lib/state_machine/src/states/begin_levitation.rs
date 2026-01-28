use crate::{state::State, state_machine::StateMachine};
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
            Event::LevitationStarted { from } => {
                info!("Levitation has started on board: {} ", from);
                EVENT_BUS
                    .sender()
                    .send(Event::RetractLateralSuspensionCommand)
                    .await;
            }
            Event::LateralSuspensionRetracted { from } => {
                info!(
                    "Lateral suspension retracted: board={} at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
            }
            Event::DynamicsStatus {
                from,
                actuator_pressure_bar,
            } => {
                info!(
                    "Dynamics Status: board={}, actuator pressure={}bar at {}ms",
                    from,
                    actuator_pressure_bar,
                    Instant::now().as_millis(),
                )
            }
            Event::LevitationStatus {
                from,
                airgap_μm,
                current_ma,
            } => {
                info!(
                    "Levitation Status: board={}, current={}mA, airgap={}μm at {}ms",
                    from,
                    current_ma.0,
                    airgap_μm,
                    Instant::now().as_millis(),
                );
            }
            Event::LevitationStable => {
                info!("Levitation stable, transitioning to Ready");
                self.transition_to(State::Ready).await;
            }
            Event::LevitationStopped { from } => {
                info!("Board={}", from)
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
