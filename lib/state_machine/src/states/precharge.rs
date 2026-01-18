use crate::{state_enum::State, state_machine::StateMachine};
use embassy_time::Instant;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

impl StateMachine {
    // --------- PRECHARGE ---------

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

                // Validate voltage - stated minimum should be close to  400V
                // So = 40000 cV target
                // Load capacitance reaches 5% of battery voltage, so allow 5% tolerance
                // TODO: Check this
                // TODO: if having voltage display in cV is annoying can change to display in mV or V
                if voltage_cv.0 < 38000 {
                    warn!("Precharge voltage too low: {}cV", voltage_cv.0);
                    self.transition_to(State::Emergency).await; // TODO Emergency or no?
                    return;
                }

                self.boards_precharged.insert(from);

                // TODO do we need to check specific boards?
                // TODO should it be == or >= here?
                if !self.desired_boards_to_charge.is_empty()
                    && self.boards_precharged.len() == self.desired_boards_to_charge.len()
                {
                    info!("Necessary boards precharged");
                    // TODO implement which boards must be precharged
                    self.transition_to(State::ReadyForLevitation).await;
                }
            }
            Event::PrechargeFailed { from, reason } => {
                info!("Board={}, reason={}", from, reason)
                // TODO decide if we need this
            }
            _ => {
                debug!("Event {} is ignored in current state", event)
            }
        }
    }
}
