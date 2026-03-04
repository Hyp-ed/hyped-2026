use crate::state::State;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

pub struct StateMachine {
    pub current_state: State,
    pub(crate) ready_for_run: bool,
    pub(crate) brakes_clamped: bool,
    pub(crate) precharge_voltage_ok: bool,
    pub(crate) discharge_voltage_ok: bool,

    // Precharge Sequence
    //  0 = all relays open, waiting for shutdown relay
    //  1 = shutdown relay closed, waiting for battery precharge relay
    //  2 = shutdown & battery precharge closed, waiting for motor controller relay
    //  3 = all relays closed
    pub(crate) precharge_step: u8,

    // Discharge Sequence
    // 0 = waiting for discharge relay to close
    // 1 = discharge relay closed, waiting for SDC relay to open
    // 2 = SDC open → transition to Idle
    pub(crate) discharge_step: u8,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: State::Idle,
            ready_for_run: false,
            brakes_clamped: true,
            precharge_step: 0,
            discharge_step: 0,
            precharge_voltage_ok: false,
            discharge_voltage_ok: false,
        }
    }

    // Actions when transitioning state
    pub(crate) async fn transition_to(&mut self, new_state: State) {
        info!("Transitioning: {:?} -> {:?}", self.current_state, new_state);
        self.current_state = new_state;
        self.entry().await;
    }

    // Entry, match on state
    pub async fn entry(&mut self) {
        info!("Entering State: {:?}", self.current_state);

        match self.current_state {
            State::Idle => self.entry_idle().await,
            State::Precharge => self.entry_precharge().await,
            State::ReadyForPropulsion => self.entry_ready_for_propulsion().await,
            State::Accelerate => self.entry_accelerate().await,
            State::Brake => self.entry_brake().await,
            State::Stopped => self.entry_stopped().await,
            State::Emergency => self.entry_emergency().await,
        }
    }

    pub async fn react(&mut self, event: Event) {
        debug!("React: {:?} in state {:?}", event, self.current_state);

        match event {
            // Emergency
            Event::Emergency { from, reason } => {
                warn!("EMERGENCY: from {:?} reason={}", from, reason);
                self.transition_to(State::Emergency).await;
                return;
            }

            // Global events
            Event::Heartbeat { from } => {
                debug!("Heartbeat from {:?}", from);
            }

            _ => {}
        }

        match self.current_state {
            State::Idle => self.react_idle(event).await,
            State::Precharge => self.react_precharge(event).await,
            State::ReadyForPropulsion => self.react_ready_for_propulsion(event).await,
            State::Accelerate => self.react_accelerate(event).await,
            State::Brake => self.react_brake(event).await,
            State::Stopped => self.react_stopped(event).await,
            State::Emergency => self.react_emergency(event).await,
        }
    }
}

#[embassy_executor::task]
pub async fn run(mut sm: StateMachine) -> ! {
    let rx = EVENT_BUS.receiver();

    sm.entry().await;

    loop {
        let ev = rx.receive().await;
        sm.react(ev).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_defaults() {
        let sm = StateMachine::new();
        assert!(matches!(sm.current_state, State::Idle));
        assert!(!sm.ready_for_run);
        assert!(sm.brakes_clamped); // brakes start clamped
        assert!(!sm.precharge_voltage_ok);
        assert!(!sm.discharge_voltage_ok);
        assert_eq!(sm.precharge_step, 0);
        assert_eq!(sm.discharge_step, 0);
    }
}
