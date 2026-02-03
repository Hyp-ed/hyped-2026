use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    pub(crate) async fn entry_precharge(&mut self) {
        info!("Starting precharge");
        EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
        // TODO reminder: include motor controller in precharge
    }

    pub(crate) async fn react_precharge(&mut self, event: Event) {
        match event {
            Event::PrechargeStarted { from } => {
                info!(
                    "Board {:?} started precharge at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
            }
            Event::PrechargeComplete { from, voltage_cv } => {
                info!(
                    "Board {:?} completed precharge at {}ms",
                    from,
                    Instant::now().as_millis(),
                );

                // Validate voltage: minimum should be close to 400V (40000 cV target)
                // Load capacitance reaches 5% of battery voltage, so allow 5% tolerance
                // TODO: Check this with electronics
                if voltage_cv.0 < 38000 {
                    warn!("Precharge voltage too low: {}cV", voltage_cv.0);
                    self.transition_to(State::Emergency).await; // TODO: Emergency or no?
                    return;
                }

                // TODO Logic: Check relays activate in this order
                // 1. Shutdown relay
                // 2. Battery precharge
                // 3. Motor controller precharge
                // 4. If discharge -> trigger emergency
                //let _ = self.boards_precharged.insert(from);

                //if !self.desired_boards_to_charge.is_empty()
                //  && self.boards_precharged.len() >= self.desired_boards_to_charge.len()
                {
                    info!("Necessary boards precharged");
                    // TODO: implement which boards must be precharged
                    self.ready_for_run = true;
                }
            }
            Event::StartRunOperatorCommand => {
                if self.ready_for_run {
                    info!("Starting Propulsion run");
                    self.transition_to(State::ReadyForPropulsion).await;
                }
            }
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY STOP PRESSED");
                self.transition_to(State::Emergency).await;
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
