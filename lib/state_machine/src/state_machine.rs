use crate::states::State;
use hyped_core::logging::{info, warn, debug};
use heapless::FnvIndexSet; 
use hyped_communications::boards::Board; 
use crate::events::Event;
use hyped_communications::{
    //actions::{Command, CommandTarget, Instruction, OUTGOING_COMMANDS},
    bus::EVENT_BUS,
    events::Event,
};

pub struct StateMachine {
    pub current_state: State,
    boards_calibrated: FnvIndexSet<Board>,
    boards_precharged: FnvIndexSet<Board>,
    desired_boards_precharged: FnvIndexSet<Board>,
    total_boards: u8,
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
            boards_calibrated: FnvIndexSet::new(),
            boards_precharged: FnvIndexSet::new(),
            // TODO implement which boards actually need to precharge

            desired_boards_precharged: FnvIndexSet::new(),
            total_boards: 5,
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



// --------- State Specific Methods ---------


// --------- IDLE --------- 

async fn entry_idle(&mut self) {
    info! ("Pod is idle")
}

async fn react_idle(&mut self, event: Event) {
    match event {
        _ => {}
    }
}

// --------- CALIBRATE --------- 

async fn entry_calibrate(&mut self) {
    info! ("Starting calibration");
    // Reset tracking
    self.boards_calibrated.clear();
    // Tell boards to start calibration
    EVENT_BUS.sender().send(Event::StartCalibrationCommand).await;
}

async fn react_calibrate(&mut self, event: Event) {
    match event {
        Event::CalibrationComplete { from } => {
            info!("Board {:?} calibrated", board);
            // Track which boards are calibrated
            self.boards_calibrated.insert(board);

            // Check if all are done
            if self.boards_calibrated.len() >= self.total_boards as usize {
                info!("All boards calibrated");
                self.transition_to(State::Precharge).await;
            }
        }
        // Check if failure event necessary
        _ => {}
    }
}


// --------- PRECHARGE --------- 

async fn entry_precharge(&mut self) {
    info! ("Starting precharge")
    EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
    // TODO reminder: include motor controller in precharge 
}

async fn react_precharge(&mut self, event: Event) {
    match event {
        Event::PrechargeStarted { from, timestamp_ms } => {
            info! ("Board {:?} started precharge at {}ms",from, timestamp_ms)
        },
        Event::PrechargeComplete { from, timestamp_ms, voltage_final_mv } => {
            info! ("Board {:?} completed precharge at {}ms",from, timestamp_ms)
            self.boards_precharged.insert(from)

            // TODO do we need to check specific boards?
            if self.boards_calibrated .len()== desired_boards_precharged.len() {
                info! ("Necessary boards precharged")
                // TODO implement which boards must be precharged
                self.transition_to(State::ReadyForLevitation).await
            }
        },
        Event::PrechargeFailed { reason, voltage_mv } => {
            // TODO decide if we need this 
        }
        _ => {}
    }
}


// --------- READY FOR LEVITATION --------- 

async fn entry_ready_for_levitation(&mut self) {
    info! ("Pod is ready for levitation")
    EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
}

async fn react_ready_for_levitation(&mut self, event: Event) {
    match event {
        Event::LevitationSystemsReady{ current_airgap_mm, current_ma } => {
            info! ("Status: current airgap: {:?}, current: {}",current_airgap_mm, current_ma)
            // TODO implement logic 
        },
        _ => {}
    }
}


// --------- BEGIN LEVITATION --------- 

async fn entry_begin_levitation(&mut self) {}
async fn react_begin_levitation(&mut self, event: Event) {}


// --------- READY --------- 

async fn entry_ready(&mut self) {}
async fn react_readt(&mut self, event: Event) {}


// --------- ACCELERATE --------- 

async fn entry_accelerate(&mut self) {}
async fn react_accelerate(&mut self, event: Event) {}


// --------- BRAKE --------- 

async fn entry_brake(&mut self) {}
async fn react_brake(&mut self, event: Event) {}


// --------- STOP LEVITATION --------- 

async fn entry_stop_levitation(&mut self) {}
async fn react_stop_levitation(&mut self, event: Event) {}


// --------- STOPPED --------- 

async fn entry_stopped(&mut self) {}
async fn react_stopped(&mut self, event: Event) {}


// --------- EMERGENCY --------- 

async fn entry_emergency(&mut self) {
    warn!("EMERGENCY STATE ENTERED");
    EVENT_BUS.sender().send(Event::EmergencyStopCommand).await;
}

async fn react_emergency(&mut self, event: Event) {
    // TODO decide what to do here 
}