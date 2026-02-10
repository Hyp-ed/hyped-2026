use crate::boards::Board;
pub use crate::emergency::Reason;
use hyped_core::types::{Current, Force, Pressure, Temperature, Velocity, Voltage};

#[derive(Debug, Clone, defmt::Format)]
pub enum Event {
    // ------ Operator Commands ------
    EmergencyStopOperatorCommand,
    PrechargeOperatorCommand,  // Idle -> Precharge
    AccelerateOperatorCommand, // ReadyForPropulsion -> Accelerate
    BrakeOperatorCommand,      // Accelerate -> Brake
    StartRunOperatorCommand,   // Start a run (Propulsion Only)

    // ------ Emergency Events ------
    Emergency {
        from: Board,
        reason: Reason,
    },

    // ------ Status Events ------
    Heartbeat {
        from: Board,
    },

    // ------ Electronics ------

    // Commands from FSM
    StartPrechargeCommand,
    StartDischargeCommand,

    // Confirmation
    PrechargeStarted,
    DischargeStarted,

    // Completion
    PrechargeComplete,
    DischargeComplete,

    VoltageStatus {
        voltage: Voltage,
    },

    // Voltage checks
    PrechargeVoltageOK,
    DischargeVoltageOK,

    // Relays
    ShutdownCircuitryRelayOpen,
    ShutdownCircuitryRelayClosed,
    BatteryPrechargeRelayOpen,
    BatteryPrechargeRelayClosed,
    MotorControllerRelayOpen,
    MotorControllerRelayClosed,
    DischargeRelayOpen,
    DischargeRelayClosed,

    // ------ Navigation ------
    EndOfTrackBrakeCommand,

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
}
