use crate::boards::Board;
pub use crate::emergency::Reason;

//use hyped_state_machine::state_enum::State;

/// Nature classification for events (compact codes).
// #[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
// #[repr(u8)]
// pub enum Nature {
//     MinorChange = 0,
//     MajorChange = 1,
//     DirEmergency = 2,
// }

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Airgap(pub u16); // micrometers

// Calculate absolute distance between two airgaps
impl Airgap {
    pub fn distance_to(&self, other: Airgap) -> u16 {
        if self.0 > other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Current(pub u16); // milliamps

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Voltage(pub u16); // centivolts

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Pressure(pub u16); // bar

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Velocity(pub u16); // km/h

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Temperature(pub u8); // celsius

// #[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
// pub struct Frequency(pub u16); // hertz

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Force(pub u16); // newtons

#[derive(Debug, Clone, defmt::Format)]
pub enum Event {
    // ------ Operator Commands ------
    EmergencyStopOperatorCommand,
    CalibrateOperatorCommand,       // Idle -> Calibrating
    BeginLevitationOperatorCommand, // Ready for levitation -> Begin Levitation
    AccelerateOperatorCommand,      // Ready -> Accelerate
    BrakeOperatorCommand,           // Accelerate -> Brake
    StopLevitationOperatorCommand,  // Brake -> Stop Levitating

    // ------ Emergency Events ------
    Emergency {
        from: Board,
        reason: Reason,
    },

    // ------ Status Events ------
    Heartbeat {
        from: Board,
    },
    // TODO any others?

    // ------ Calibration ------
    StartCalibrationCommand,
    CalibrationComplete {
        from: Board,
    },

    // ------ Electronics ------

    // Commands from FSM
    StartPrechargeCommand,
    StartDischargeCommand,

    // Confirmation
    PrechargeStarted {
        from: Board,
    },
    DischargeStarted {
        from: Board,
    },

    // Completion
    PrechargeComplete {
        from: Board,
        voltage_cv: Voltage,
    },
    DischargeComplete {
        from: Board,
        voltage_cv: Voltage,
    },

    // Failure
    PrechargeFailed {
        from: Board,
        reason: Reason,
    },

    // ------ Levitation ------

    // Ready check
    LevitationSystemsReady,

    // Commands from FSM
    StartLevitationCommand,
    StopLevitationCommand,

    // Confirmation
    LevitationStarted {
        from: Board,
    },

    // Continuous status updates
    LevitationStatus {
        from: Board,
        airgap_μm: Airgap,
        current_ma: Current,
    },

    // Completion
    LevitationStopped {
        from: Board,
    },

    // Failure
    LevitationFailed {
        from: Board,
        reason: Reason,
    },

    // ------ Dynamics ------

    // Commands from FSM
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,

    // Completion
    BrakesClamped {
        from: Board,
    },

    BrakesUnclamped {
        from: Board,
    },

    LateralSuspensionRetracted {
        from: Board,
    },

    LateralSuspensionExtended {
        from: Board,
    },
    DynamicsStatus {
        from: Board,
        actuator_pressure_bar: Pressure,
    },

    // TODO Failure events?

    // ------ Propulsion ------

    // Commands from FSM
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,

    // Confirmation
    PropulsionAccelerationStarted,
    PropulsionBrakingStarted,

    // Continuous status updates
    PropulsionStatus {
        current_ma: Current,
        velocity_kmh: Velocity,
        temperature_c: Temperature,
        voltage_cv: Voltage,
    },

    PropulsionForce {
        force_n: Force, // calculated thrust force
    },

    PropulsionFailed {
        from: Board,
        reason: Reason,
    },
}
