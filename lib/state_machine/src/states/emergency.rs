use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    // --------- EMERGENCY ---------

    pub(crate) async fn entry_emergency(&mut self) {
        warn!("EMERGENCY STATE ENTERED");
        EVENT_BUS.sender().send(Event::ClampBrakesCommand).await;
        EVENT_BUS
            .sender()
            .send(Event::ExtendLateralSuspensionCommand)
            .await;
        EVENT_BUS.sender().send(Event::StopLevitationCommand).await;
        EVENT_BUS
            .sender()
            .send(Event::StartPropulsionBrakingCommand)
            .await;
    }

    pub(crate) async fn react_emergency(&mut self, event: Event) {
        match event {
            Event::BrakesClamped {
                actuator_pressure_bar,
            } => {
                info!(
                    "Emergency brakes engaged: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
                    Instant::now().as_millis(),
                );
            }

            Event::LateralSuspensionExtended {
                actuator_pressure_bar,
            } => {
                info!(
                    "Emergency suspension extended: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
                    Instant::now().as_millis(),
                );
            }
            // TODO decide what to do here
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
