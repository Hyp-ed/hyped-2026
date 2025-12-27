use crate::states::State;
use hyped_core::logging::{info, warn, debug};
use hyped_communications::{
    //actions::{Command, CommandTarget, Instruction, OUTGOING_COMMANDS},
    bus::EVENT_BUS,
    events::Event,
};

pub struct StateMachine {
    pub current_state: State,
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
        }
    }

    // Actions when transitioning state 
async fn transition_to(&mut self, new_state: State) {
    info!("Transitioning: {:?} -> {:?}", self.current_state, new_state);
    self.current_state = new_state;
    self.entry().await;  
    // Todo can add validation later if needed
}

// Entry, match on state 
pub async fn entry(&mut self) {
    info!("Entering State: {:?}", self.current_state);
    
    match self.current_state {
        State::Idle => self.entry_idle().await,
        State::Calibrate => self.entry_calibrate().await,
        State::Precharge => self.entry_precharge().await,
        State::ReadyForLevitation => self.entry_ready_for_levitation().await,
        State::BeginLevitation => self.entry_begin_levitation().await,
        State::Ready => self.entry_ready().await,
        State::Accelerate => self.entry_accelerate().await,
        State::Brake => self.entry_brake().await,
        State::StopLevitation => self.entry_stop_levitation().await,
        State::Stopped => self.entry_stopped().await,
        State::Emergency => self.entry_emergency().await,
    }
}

pub async fn react(&mut self, event: Event) {
    debug!("React: {:?} in state {:?}", event, self.current_state);

    match event {
        // Emergency
        Event::Emergency{ from, reason } => {
            warn!("EMERGENCY: from {:?} reason={}",from, reason);
            self.transition_to(State::Emergency).await;
            return;
        },
        
        // Global events
        Event::Heartbeat { from } => {
            // Todo: handle or ignore
            // Update last seen or something
    },

    _ => {}
}

    match self.current_state {
        State::Idle => self.react_idle(event).await,
        State::Calibrate => self.react_calibrate(event).await,
        State::Precharge => self.react_precharge(event).await,
        State::ReadyForLevitation => self.react_ready_for_levitation(event).await,
        State::BeginLevitation => self.react_begin_levitation(event).await,
        State::Ready => self.react_ready(event).await,
        State::Accelerate => self.react_accelerate(event).await,
        State::Brake => self.react_brake(event).await,
        State::StopLevitation => self.react_stop_levitation(event).await,
        State::Stopped => self.react_stopped(event).await,
        State::Emergency => self.react_emergency(event).await,
    }
}
}

#[embassy_executor::task]
pub async fn run(mut sm: StateMachine) -> ! {
    let mut rx = EVENT_BUS.receiver();

    sm.entry().await;

    loop {
        let ev = rx.receive().await;
        sm.react(ev).await;
    
    }
}