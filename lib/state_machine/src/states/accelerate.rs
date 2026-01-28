use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_accelerate(&mut self) {
        info!("Pod is accelerating");
        EVENT_BUS
            .sender()
            .send(Event::StartPropulsionAccelerationCommand)
            .await;
    }
    pub(crate) async fn react_accelerate(&mut self, event: Event) {
        match event {
            // Braking
            Event::BrakeOperatorCommand => {
                info!("Operator initiated braking");
                self.transition_to(State::Brake).await;
            }
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            // TODO: add an event for end of track braking
            // Status
            Event::PropulsionAccelerationStarted => {
                info!("Acceleration started at {}ms", Instant::now().as_millis());
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
            }
            Event::PropulsionForce { force_n } => {
                info!(
                    "
                Calculated propulsion force: {}N",
                    force_n.0
                )
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
