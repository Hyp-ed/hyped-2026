use crate::states::State;
use hyped_communications::{
    actions::{Command, CommandTarget, Instruction, OUTGOING_COMMANDS},
    bus::EVENT_BUS,
    events::Event,
};
use hyped_core::logging::{info, warn};

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
        StateMachine {
            current_state: State::Idle,
        }
    }

    /// Handles a transition from the current state to the given state.
    /// If the state transition is valid, the state machine will transition to the new state.
    pub fn handle_transition(&mut self, to_state: &State) -> Option<State> {
        let new_state = match (self.current_state, to_state) {
            (State::Idle, State::Calibrate) => Some(State::Calibrate),
            (State::Calibrate, State::Precharge) => Some(State::Precharge),
            (State::Precharge, State::ReadyForLevitation) => Some(State::ReadyForLevitation),
            (State::ReadyForLevitation, State::BeginLevitation) => Some(State::BeginLevitation),
            (State::BeginLevitation, State::Ready) => Some(State::Ready),
            (State::Ready, State::Accelerate) => Some(State::Accelerate),
            (State::Accelerate, State::Brake) => Some(State::Brake),
            (State::Accelerate, State::Emergency) => Some(State::Emergency),
            (State::Brake, State::StopLevitation) => Some(State::StopLevitation),
            (State::StopLevitation, State::Stopped) => Some(State::Stopped),
            (State::Stopped, State::Idle) => Some(State::Idle),
            _ => None,
        };

        match new_state {
            Some(new_state) => {
                info!(
                    "Transitioning from {:?} to {:?}",
                    self.current_state, new_state
                );
                self.current_state = new_state;
                Some(new_state)
            }
            None => {
                warn!(
                    "Invalid transition requested from {:?} to {:?}",
                    self.current_state, to_state
                );
                None
            }
        }
    }

    fn try_transition(&mut self, to: State) -> bool {
        self.handle_transition(&to).is_some()
    }
}

#[embassy_executor::task]
pub async fn run(mut sm: StateMachine) -> ! {
    let mut rx = EVENT_BUS.receiver();
    let mut tx = OUTGOING_COMMANDS.sender();

    loop {
        let ev = rx.receive().await;

        match ev {
            // Prioritise emergency events
            Event::Emergency { from, reason } => {
                if sm.current_state != State::Emergency {
                    info!("Emergency reported by {:?} (reason={:?})", from, reason);
                    if sm.try_transition(State::Emergency) {
                        let command = Command {
                            target: CommandTarget::AllBoards,
                            instruction: Instruction::EmergencyStop { reason },
                        };
                        let _ = tx.send(command).await;
                    } else {
                        warn!(
                            "Failed to transition to Emergency on incoming alert from {:?}",
                            from
                        );
                    }
                }
            }

            Event::RequestTransition {
                from,
                to,
                by,
                reason,
            } => {
                info!(
                    "Transition request from {:?} via {:?}: {:?} (reason={:?})",
                    from, by, to, reason
                );

                if sm.try_transition(to) {
                    let command = Command {
                        target: CommandTarget::AllBoards,
                        instruction: Instruction::EnterState(to),
                    };
                    let _ = tx.send(command).await;
                } else {
                    warn!(
                        "Rejected transition request {:?} -> {:?} from {:?}",
                        sm.current_state, to, from
                    );
                }
            }
        }
    }
}