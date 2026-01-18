use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    // --------- STOPPED ---------

    pub(crate) async fn entry_stopped(&mut self) {
        info!("Pod is stopped");
        EVENT_BUS.sender().send(Event::StartDischargeCommand).await;
    }
    pub(crate) async fn react_stopped(&mut self, event: Event) {
        match event {
            Event::DischargeStarted { from } => {
                info!(
                    "Board {:?} started discharge at {}ms",
                    from,
                    Instant::now().as_millis(),
                );
            }
            Event::DischargeComplete { from, voltage_cv } => {
                info!(
                    "Board {:?} completed discharge at {}ms with a final voltage of {}cV",
                    from,
                    Instant::now().as_millis(),
                    voltage_cv,
                );
                let _ = self.boards_discharged.insert(from);

                // TODO do we need to check specific boards?
                // TODO should it be == or >= here?
                // Can use precharged, since its the same
                // ITODO check: is is only electronics that discharges, or also motor controller?
                if !self.desired_boards_to_charge.is_empty()
                    && self.boards_discharged.len() == self.desired_boards_to_charge.len()
                {
                    info!("Necessary boards discharged");
                    // TODO implement which boards must be discharged
                    self.transition_to(State::Idle).await;
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
