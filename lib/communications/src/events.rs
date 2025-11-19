use crate::boards::Board;
use hyped_state_machine::states::State;

/// Nature classification for events (compact codes).
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
#[repr(u8)]
pub enum Nature {
    MinorChange = 0,
    MajorChange = 1,
    DirEmergency = 2,
}

/// Per-state event envelopes (extend payload enums as needed).
#[derive(Debug, Clone, defmt::Format)]
pub enum StateEvent {
    Idle(IdleEvent),
    Calibrate(CalibrateEvent),
    Precharge(PrechargeEvent),
    Levitation(LevitationEvent),
    Ready(ReadyEvent),
    Accelerate(AccelerateEvent),
    Brake(BrakeEvent),
    Emergency(EmergencyEvent),
}

#[derive(Debug, Clone, defmt::Format)]
pub enum IdleEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum CalibrateEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum PrechargeEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum LevitationEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum ReadyEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum AccelerateEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum BrakeEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum EmergencyEvent {}

#[derive(Debug, Clone, defmt::Format)]
pub enum Event {
    Emergency {
        from: Board,
        reason: u8,
    },
    EnterState(State),
    ExitState(State),
    StateEvent(StateEvent),
}