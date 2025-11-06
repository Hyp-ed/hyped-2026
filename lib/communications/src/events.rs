use crate::boards::Board;
use hyped_state_machine::states::State;

/// Nature classification for events (compact codes).
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
#[repr(u8)]
pub enum Nature {
    MinorChange = 0,
    MajorChange = 1,
    WarningState = 2, // warnings and emergencies can be treated similarly by policy
    DirEmergency = 3,
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

/// Actions taken with permission
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum ActionWPerm {}

/// Direct control actions that do not require permission.
#[derive(Debug, Clone, PartialEq, defmt::Format)]
pub enum DirAction {}

#[derive(Debug, Clone, defmt::Format)]
pub enum Event {
    Emergency {
        from: Board,
        reason: u8,
    },

    CommandTransition {
        from: State,
        to: State,
    },
    HeartbeatSeen {
        from: Board,
    },

    RequestTransition {
        from: Board,
        to: State,
        by: Board,
        reason: u8,
    },

    RequestPermission {
        by: Board,
        action: ActionWPerm,
        reason: u8,
    },

    PermissionGranted {
        to: Board,
        action: ActionWPerm,
    },

    PermissionDenied {
        to: Board,
        action: ActionWPerm,
    },

    DirAction {
        board: Board,
        action: DirAction,
    },
    EnterState(State),
    ExitState(State),

    StateEvent(StateEvent),
}