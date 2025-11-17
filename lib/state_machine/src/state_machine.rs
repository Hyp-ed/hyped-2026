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
        }
    }
}