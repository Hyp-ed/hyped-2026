use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    // --------- STOP LEVITATION ---------

    pub(crate) async fn entry_stop_levitation(&mut self) {
        info!("Levitation stopping");
        info!("Extending lateral suspension");
        EVENT_BUS
            .sender()
            .send(Event::ExtendLateralSuspensionCommand)
            .await;
    }
    pub(crate) async fn react_stop_levitation(&mut self, event: Event) {
        match event {
            Event::LateralSuspensionExtended {
                actuator_pressure_bar,
            } => {
                info!(
                    "Lateral suspension extended: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
                    Instant::now().as_millis(),
                );
                EVENT_BUS.sender().send(Event::StopLevitationCommand).await;
            }
            Event::LevitationStopped { from } => {
                info!("Levitation stopped on board={}", from);
                self.transition_to(State::Stopped).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
