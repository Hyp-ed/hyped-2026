use crate::boards::Board;
pub use crate::emergency::Reason;
use hyped_core::types::{Airgap, Current, Force, Pressure, Temperature, Velocity, Voltage};

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

    // Calculated thrust force
    PropulsionForce {
        force_n: Force,
    },

    PropulsionFailed {
        from: Board,
        reason: Reason,
    },
}
