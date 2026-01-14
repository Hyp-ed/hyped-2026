use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    // --------- READY FOR LEVITATION ---------

    pub(crate) async fn entry_ready_for_levitation(&mut self) {
        info!("Pod is ready for levitation");
        EVENT_BUS.sender().send(Event::UnclampBrakesCommand).await;
    }

    pub(crate) async fn react_ready_for_levitation(&mut self, event: Event) {
        match event {
            Event::BrakesUnclamped {
                actuator_pressure_bar,
            } => {
                info!(
                    "Brakes unclamped: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
                    Instant::now().as_millis(),
                );
            }
            Event::LevitationSystemsReady => {
                info!("Levitation systems ready, awaiting operator command");
                self.levitation_systems_ready = true;
            }
            Event::BeginLevitationOperatorCommand => {
                if self.levitation_systems_ready {
                    self.transition_to(State::BeginLevitation).await;
                } else {
                    warn!("Cannot start levitation, systems not ready");
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
