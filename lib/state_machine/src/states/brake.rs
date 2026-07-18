use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::events::Event;
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_brake(&mut self) {
        info!("Pod is braking");
        self.shutdown_circuitry_relay_open = false;
        self.battery_precharge_relay_open = false;
        self.motor_controller_relay_open = false;
        self.brakes_clamped = false;
        self.motor_controller_operational = false;
        self.queue_publish(Event::OpenPrechargeRelaysCommand);
        self.queue_publish(Event::StartPropulsionBrakingCommand);
        self.queue_publish(Event::ClampBrakesCommand);
    }

    pub(crate) async fn react_brake(&mut self, event: Event) {
        match event {
            Event::PropulsionBrakingStarted => {
                info!("Braking started at {}ms", Instant::now().as_millis(),);
            }
            Event::BrakesClamped { from } => {
                info!(
                    "Brakes clamped: board={} at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
                self.brakes_clamped = true;
                self.complete_braking_if_safe().await;
            }
            Event::ShutdownCircuitryRelayOpen => {
                self.shutdown_circuitry_relay_open = true;
                self.complete_braking_if_safe().await;
            }
            Event::BatteryPrechargeRelayOpen => {
                self.battery_precharge_relay_open = true;
                self.complete_braking_if_safe().await;
            }
            Event::MotorControllerRelayOpen => {
                self.motor_controller_relay_open = true;
                self.complete_braking_if_safe().await;
            }
            Event::ShutdownCircuitryRelayClosed
            | Event::BatteryPrechargeRelayClosed
            | Event::MotorControllerRelayClosed
            | Event::BrakesUnclamped { .. }
            | Event::PropulsionAccelerationStarted => {
                warn!("Normal braking safety condition violated");
                self.transition_to(State::Emergency).await;
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

    async fn complete_braking_if_safe(&mut self) {
        if self.shutdown_circuitry_relay_open
            && self.battery_precharge_relay_open
            && self.motor_controller_relay_open
            && self.brakes_clamped
        {
            info!("Brakes deployed and HV isolated; entering Stopped");
            self.transition_to(State::Stopped).await;
        }
    }
}
