use crate::state::State;
//use heapless::FnvIndexSet;
use hyped_communications::{bus::EVENT_BUS, events::Event};
use hyped_core::logging::{debug, info, warn};

pub struct StateMachine {
    pub current_state: State,
    //pub(crate) boards_calibrated: FnvIndexSet<Board, 8>,
    //pub(crate) boards_precharged: FnvIndexSet<Board, 8>,
    //pub(crate) desired_boards_to_charge: FnvIndexSet<Board, 8>,
    //pub(crate) boards_discharged: FnvIndexSet<Board, 8>,
    //pub(crate) total_boards: u8,
    pub(crate) ready_for_run: bool,
    pub(crate) brakes_clamped: bool,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        //let desired = FnvIndexSet::new();
        // TODO: insert which boards need precharged
        // Electronics
        // Motor Controller
        // desired.insert(Board::<board>).unwrap();

        Self {
            current_state: State::Idle,
            //boards_calibrated: FnvIndexSet::new(),
            //boards_precharged: FnvIndexSet::new(),
            //boards_discharged: FnvIndexSet::new(),
            //desired_boards_to_charge: desired,
            //total_boards: 5,
            ready_for_run: false,
            brakes_clamped: true,
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
