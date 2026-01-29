use crate::{state::State, state_machine::StateMachine};
//state_enum::State,
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
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
            Event::BrakesClamped { from } => {
                info!(
                    "Emergency brakes engaged: board={} at {}ms",
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
            Event::LateralSuspensionExtended { from } => {
                info!(
                    "Emergency suspension extended: board={} at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
            }
            Event::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => {
                info!(
                    "Propulsion status: {}mA, {}km/h, {}°C, {}cV",
                    current_ma.0, velocity_kmh.0, temperature_c.0, voltage_cv.0,
                );
                info!("Braking: velocity={}km/h", velocity_kmh.0);

                // Check if stopped
                if velocity_kmh.0 == 0 {
                    info!("Pod has stopped, transitioning to Stopped");
                    self.transition_to(State::Stopped).await;
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
