use crate::state::State;
use heapless::Vec as HeaplessVec;
use hyped_communications::{bus, bus::DynSubscriber, events::Event};
use hyped_core::logging::{debug, info, warn};

const MAX_PENDING_EVENTS: usize = 8;

#[derive(Debug, PartialEq, defmt::Format, Clone, Copy)]
pub enum PrechargeStep {
    Initial,
    ShutdownClosed,
    BatteryPrechargeClosed,
    AllClosed,
}

#[derive(Debug, PartialEq, defmt::Format, Clone, Copy)]
pub enum DischargeStep {
    Initial,
    DischargeClosed,
    SdcOpen,
}

pub struct StateMachine {
    pub current_state: State,
    pub(crate) ready_for_run: bool,
    pub(crate) brakes_clamped: bool,
    pub(crate) precharge_voltage_ok: bool,
    pub(crate) discharge_voltage_ok: bool,

    pub(crate) precharge_step: PrechargeStep,
    pub(crate) discharge_step: DischargeStep,
    pending_events: HeaplessVec<Event, MAX_PENDING_EVENTS>,
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
            precharge_step: PrechargeStep::Initial,
            discharge_step: DischargeStep::Initial,
            precharge_voltage_ok: false,
            discharge_voltage_ok: false,
            pending_events: HeaplessVec::new(),
        }
    }

    pub(crate) fn queue_publish(&mut self, event: Event) {
        if self.pending_events.push(event.clone()).is_err() {
            warn!("Pending event queue full, dropping {:?}", event);
        }
    }

    pub(crate) async fn drain_pending(&mut self) {
        while let Some(event) = self.pending_events.pop() {
            bus::publish(event).await;
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
pub async fn run(mut sm: StateMachine, mut events: DynSubscriber<'static, Event>) -> ! {
    sm.entry().await;
    sm.drain_pending().await;

    loop {
        let ev = events.next_message_pure().await;
        sm.react(ev).await;
        sm.drain_pending().await;
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
        assert_eq!(sm.precharge_step, PrechargeStep::Initial);
        assert_eq!(sm.discharge_step, DischargeStep::Initial);
    }
}
