use crate::boards::Board;
use hyped_state_machine::states::State;

/// Nature classification for events (compact codes)
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
#[repr(u8)]
pub enum Nature {
    MinorChange = 0,
    MajorChange = 1,
    DirEmergency = 2,
}
#[derive(Debug, Clone, defmt::Format)]
pub enum Event {
    Emergency {
        from: Board,
        reason: u8,
    },
    EnterState(State),
    ExitState(State),
    StateEvent { nature: Nature }, // Temp for generic react() and entry()
}