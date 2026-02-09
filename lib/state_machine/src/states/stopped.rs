use crate::{state::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info};

impl StateMachine {
    pub(crate) async fn entry_stopped(&mut self) {
        info!("Pod is stopped");
        EVENT_BUS.sender().send(Event::StartDischargeCommand).await;
    }
    pub(crate) async fn react_stopped(&mut self, event: Event) {
        match event {
            Event::DischargeStarted => {
                info!("Started discharge at {}ms", Instant::now().as_millis(),);
            }
            Event::DischargeComplete => {
                info!("Completed discharge at {}ms", Instant::now().as_millis(),);
                // TODO: Check discharge relay is opened
                //let _ = self.boards_discharged.insert(from);

                //if !self.desired_boards_to_charge.is_empty()
                //   && self.boards_discharged.len() >= self.desired_boards_to_charge.len()
                {
                    info!("Necessary boards discharged");
                    // TODO: implement which boards must be discharged
                    self.transition_to(State::Idle).await;
                }
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
