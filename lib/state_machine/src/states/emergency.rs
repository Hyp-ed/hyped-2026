use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_emergency(&mut self) {
        warn!("EMERGENCY STATE ENTERED");
        self.shutdown_circuitry_relay_open = false;
        self.battery_precharge_relay_open = false;
        self.motor_controller_relay_open = false;
        self.brakes_clamped = false;
        self.motor_controller_operational = false;
        self.queue_publish(Event::OpenPrechargeRelaysCommand);
        self.queue_publish(Event::ClampBrakesCommand);
        self.queue_publish(Event::StartPropulsionBrakingCommand);
    }

    pub(crate) async fn react_emergency(&mut self, event: Event) {
        match event {
            Event::BrakesClamped { from } => {
                self.brakes_clamped = true;
                info!(
                    "Emergency brakes engaged: board={} at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
            }
            Event::ShutdownCircuitryRelayOpen => {
                self.shutdown_circuitry_relay_open = true;
            }
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
            }
            Event::ResetEmergencyOperatorCommand => {
                if self.shutdown_circuitry_relay_open
                    && self.battery_precharge_relay_open
                    && self.motor_controller_relay_open
                    && self.brakes_clamped
                {
                    info!("Emergency reset accepted after safe-state confirmation");
                    self.transition_to(State::Idle).await;
                } else {
                    warn!("Emergency reset rejected: safe-state confirmations incomplete");
                }
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

                // Emergency remains latched even after the pod stops.
            }

            Event::EmergencyStopOperatorCommand => {
                info!("Pod already in emergency, ignoring command")
            }

            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
