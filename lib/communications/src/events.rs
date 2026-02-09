use crate::boards::Board;
pub use crate::emergency::Reason;
use hyped_core::types::{Airgap, Current, Force, Pressure, Temperature, Velocity, Voltage};

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
    PrechargeStarted,
    DischargeStarted,

    // Completion
    PrechargeComplete,
    DischargeComplete,

    PrechargeVoltageOK,
    // Validate voltage: minimum should be close to 400V (40000 cV target)
    // Load capacitance reaches 5% of battery voltage, so allow 5% tolerance
    // TODO: Check this with electronics
    // if voltage_cv.0 < 38000 {
    //                 warn!("Precharge voltage too low: {}cV", voltage_cv.0);
    //                 self.transition_to(State::Emergency).await; // TODO: Emergency or no?
    //                 return;
    //             }

    // Relays
    ShutdownCircuitryRelayOpen,
    ShutdownCircuitryRelayClosed,
    BatteryPrechargeRelayOpen,
    BatteryPrechargeRelayClosed,
    MotorControllerRelayOpen,
    MotorControllerRelayClosed,
    DischargeRelayOpen,
    DischargeRelayClosed,

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

    LevitationStable,

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
