use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    // --------- BRAKE ---------

    pub(crate) async fn entry_brake(&mut self) {
        info!("Pod is braking");
        EVENT_BUS
            .sender()
            .send(Event::StartPropulsionBrakingCommand)
            .await;
        EVENT_BUS.sender().send(Event::ClampBrakesCommand).await;
    }

    pub(crate) async fn react_brake(&mut self, event: Event) {
        match event {
            Event::PropulsionBrakingStarted => {
                info!("Braking started at {}ms", Instant::now().as_millis(),);
            }
            Event::BrakesClamped {
                actuator_pressure_bar,
            } => {
                info!(
                    "Brakes clamped: pressure={}bar at {}ms",
                    actuator_pressure_bar.0,
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
                    info!("Pod has stopped, ready for stop levitation command");
                    // TODO: auto transition or wait for operator?
                }
            }
            Event::PropulsionForce { force_n } => {
                info!(
                    "
                Calculated propulsion force: {}N",
                    force_n.0
                )
            }
            Event::StopLevitationOperatorCommand => {
                info!("Stop levitation pressed");
                self.transition_to(State::StopLevitation).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
