use crate::boards::Board;
//use hyped_state_machine::states::State;

/// Nature classification for events (compact codes).
// #[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
// #[repr(u8)]
// pub enum Nature {
//     MinorChange = 0,
//     MajorChange = 1,
//     DirEmergency = 2,
// }

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Airgap(pub u32); // millimeters

// Calculate absolute distance two airgaps
impl Airgap {
    pub fn distance_to(&self, other: Airgap) -> u32 {
        if self.0 > other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        }
    }
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Current(pub u32); // milliamps

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Timestamp(pub u64); // milliseconds

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Voltage(pub u32); // millivolts

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Pressure(pub u16); // bar

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Velocity(pub u16); // km/h

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Temperature(pub u8); // celsius

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Frequency(pub u16); // hertz

#[derive(Debug, Clone, Copy, defmt::Format)]
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
        reason: u8,
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
        timestamp_ms: Timestamp,
    },
    DischargeStarted {
        from: Board,
        timestamp_ms: Timestamp,
    },

    // Completion
    PrechargeComplete {
        from: Board,
        timestamp_ms: Timestamp,
        voltage_final_mv: Voltage,
    },
    DischargeComplete {
        from: Board,
        timestamp_ms: Timestamp,
        voltage_final_mv: Voltage,
    },

    // TODO do we want this?
    // Failure
    PrechargeFailed {
        reason: u8,
        voltage_mv: Voltage,
    },

    // ------ Levitation ------

    // Ready check
    LevitationSystemsReady {
        ready: bool,
        // Send values if not ready
        // check if we need this
        current_airgap_mm: Airgap,
        current_ma: Current,
    },

    // Commands from FSM
    StartLevitationCommand,
    StopLevitationCommand,

    // Confirmation
    LevitationStarted {
        initial_current_ma: Current,
        initial_airgap_mm: Airgap,
        target_airgap_mm: Airgap,
    },

    // Continuous status updates
    LevitationStatus {
        current_airgap_mm: Airgap,
        target_airgap_mm: Airgap,
        current_ma: Current,
    },

    // Completion
    LevitationStopped {
        final_airgap_mm: Airgap,
        final_current_ma: Current,
    },

    // TODO If we want to handle rather than trigger shutdown?
    // Failure
    LevitationFailed {
        reason: u8,
        current_airgap_mm: Airgap,
        current_ma: Current,
    },

    // ------ Dynamics ------

    // Commands from FSM
    UnclampBrakesCommand,
    ClampBrakesCommand,
    RetractLateralSuspensionCommand,
    ExtendLateralSuspensionCommand,

    // Completion
    BrakesClamped {
        actuator_pressure_bar: Pressure,
        timestamp_ms: Timestamp,
    },

    BrakesUnclamped {
        actuator_pressure_bar: Pressure,
        timestamp_ms: Timestamp,
    },

    LateralSuspensionRetracted {
        actuator_pressure_bar: Pressure,
        timestamp_ms: Timestamp,
    },

    LateralSuspensionExtended {
        actuator_pressure_bar: Pressure,
        timestamp_ms: Timestamp,
    },

    // TODO Failure events?

    // ------ Propulsion ------

    // Commands from FSM
    StartPropulsionAccelerationCommand,
    StartPropulsionBrakingCommand,

    // Confirmation
    PropulsionAccelerationStarted {
        timestamp_ms: Timestamp,
    },
    PropulsionBrakingStarted {
        timestamp_ms: Timestamp,
    },

    // Continuous status updates
    PropulsionStatus {
        current_ma: Current,
        velocity_kmh: Velocity,
        temperature_c: Temperature,
        voltage_mv: Voltage,
        frequency_hz: Frequency,
        force_n: Force, // calculated thrust force
    },
    // TODO decide whether to handle failure here or as an emergency
}
