use crate::states::State;
use hyped_core::logging::{info, warn, debug};
use heapless::FnvIndexSet; 
use hyped_communications::boards::Board; 
use hyped_communications::{
    //actions::{Command, CommandTarget, Instruction, OUTGOING_COMMANDS},
    bus::EVENT_BUS,
    events::Event,
};

pub struct StateMachine {
    pub current_state: State,
    boards_calibrated: FnvIndexSet<Board>,
    boards_precharged: FnvIndexSet<Board>,
    desired_boards_to_charge: FnvIndexSet<Board>,
    boards_discharged: FnvIndexSet<Board>,
    total_boards: u8,
    levitation_systems_ready: bool,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        let mut desired = FnvIndexSet::new();
        // TODO: insert which boards need precharged
        // Electronics
        // Motor Controller? (stated in requirements needs ~400 volts)
        // desired.insert(Board::<board>).unwrap();

        Self {
            current_state: State::Idle,
            boards_calibrated: FnvIndexSet::new(),
            boards_precharged: FnvIndexSet::new(),
            boards_discharged: FnvIndexSet::new(),
            desired_boards_to_charge: desired,
            total_boards: 5,
            levitation_systems_ready: false,
        }
    }

    // Actions when transitioning state 
async fn transition_to(&mut self, new_state: State) {
    info!("Transitioning: {:?} -> {:?}", self.current_state, new_state);
    self.current_state = new_state;
    self.entry().await;  
    // Todo can add validation later if needed
}

// Entry, match on state 
pub async fn entry(&mut self) {
    info!("Entering State: {:?}", self.current_state);
    
    match self.current_state {
        State::Idle => self.entry_idle().await,
        State::Calibrate => self.entry_calibrate().await,
        State::Precharge => self.entry_precharge().await,
        State::ReadyForLevitation => self.entry_ready_for_levitation().await,
        State::BeginLevitation => self.entry_begin_levitation().await,
        State::Ready => self.entry_ready().await,
        State::Accelerate => self.entry_accelerate().await,
        State::Brake => self.entry_brake().await,
        State::StopLevitation => self.entry_stop_levitation().await,
        State::Stopped => self.entry_stopped().await,
        State::Emergency => self.entry_emergency().await,
    }
}

pub async fn react(&mut self, event: Event) {
    debug!("React: {:?} in state {:?}", event, self.current_state);

    match event {
        // Emergency
        Event::Emergency{ from, reason } => {
            warn!("EMERGENCY: from {:?} reason={}",from, reason);
            self.transition_to(State::Emergency).await;
            return;
        },
        
        // Global events
        Event::Heartbeat { from } => {
            debug!("Heartbeat from {:?}", from);
    },

    _ => {}
}

    match self.current_state {
        State::Idle => self.react_idle(event).await,
        State::Calibrate => self.react_calibrate(event).await,
        State::Precharge => self.react_precharge(event).await,
        State::ReadyForLevitation => self.react_ready_for_levitation(event).await,
        State::BeginLevitation => self.react_begin_levitation(event).await,
        State::Ready => self.react_ready(event).await,
        State::Accelerate => self.react_accelerate(event).await,
        State::Brake => self.react_brake(event).await,
        State::StopLevitation => self.react_stop_levitation(event).await,
        State::Stopped => self.react_stopped(event).await,
        State::Emergency => self.react_emergency(event).await,
    }
}
}

#[embassy_executor::task]
pub async fn run(mut sm: StateMachine) -> ! {
    let mut rx = EVENT_BUS.receiver();

    sm.entry().await;

    loop {
        let ev = rx.receive().await;
        sm.react(ev).await;
    
    }
}


impl StateMachine {
// --------- State Specific Methods ---------


// --------- IDLE --------- 

async fn entry_idle(&mut self) {
    info! ("Pod is idle");
}

async fn react_idle(&mut self, event: Event) {
    match event {
        Event::CalibrateOperatorCommand => {
            info!("Calibrate command received");
            self.transition_to(State::Calibrate).await;
        }
        _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}

// --------- CALIBRATE --------- 

async fn entry_calibrate(&mut self) {
    info! ("Starting calibration");
    // Reset tracking
    self.boards_calibrated.clear();
    // Tell boards to start calibration
    EVENT_BUS.sender().send(Event::StartCalibrationCommand).await;
}

async fn react_calibrate(&mut self, event: Event) {
    match event {
        Event::CalibrationComplete { from } => {
            info!("Board {:?} calibrated", from);
            // Track which boards are calibrated
            self.boards_calibrated.insert(from);

            // Check if all are done
            if self.boards_calibrated.len() >= self.total_boards as usize {
                info!("All boards calibrated");
                self.transition_to(State::Precharge).await;
            }
        }
        // Check if failure event necessary
         _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- PRECHARGE --------- 

async fn entry_precharge(&mut self) {
    info! ("Starting precharge");
    EVENT_BUS.sender().send(Event::StartPrechargeCommand).await;
    // TODO reminder: include motor controller in precharge 
}

async fn react_precharge(&mut self, event: Event) {
    match event {
        Event::PrechargeStarted { from, timestamp_ms } => {
            info! ("Board {:?} started precharge at {}ms",from, timestamp_ms.0);
        },
        Event::PrechargeComplete { from, timestamp_ms, voltage_final_mv } => {
            info! ("Board {:?} completed precharge at {}ms",from, timestamp_ms.0);

            // Validate voltage - stated minimum should be close to  400V 
            // So = 400000mV target
            // Load capacitance reaches 5% of battery voltage, so allow 5% tolerance
            if voltage_final_mv.0 < 380000 { 
                warn!("Precharge voltage too low: {}mV", voltage_final_mv.0);
                self.transition_to(State::Emergency).await; // TODO Emergency or no?
                return;
            }

            self.boards_precharged.insert(from);

            // TODO do we need to check specific boards?
            // TODO should it be == or >= here? 
            if !self.desired_boards_to_charge.is_empty() && self.boards_precharged.len() == self.desired_boards_to_charge.len() {
                info! ("Necessary boards precharged");
                // TODO implement which boards must be precharged
                self.transition_to(State::ReadyForLevitation).await;
            }
        },
        Event::PrechargeFailed { reason, voltage_mv } => {
            // TODO decide if we need this 
        }
        _ => {debug! ("Event {} is ignored in current state",event)}
    }
}


// --------- READY FOR LEVITATION --------- 

async fn entry_ready_for_levitation(&mut self) {
    info! ("Pod is ready for levitation");
    EVENT_BUS.sender().send(Event::UnclampBrakesCommand).await;
}

async fn react_ready_for_levitation(&mut self, event: Event) {
    match event {
        Event::BrakesUnclamped { actuator_pressure_bar, timestamp_ms } => {
            info!("Brakes unclamped: pressure={}bar at {}ms",
            actuator_pressure_bar.0, timestamp_ms.0);
        },
        Event::LevitationSystemsReady{ ready, current_airgap_mm, current_ma } => {
            if ready {
                  info!("Levitation systems ready, awaiting operator command");
                  self.levitation_systems_ready = true;
              } else {
                  warn!("Levitation systems not ready");
                  info! ("Status: current airgap: {:?}mm, current: {}ma",current_airgap_mm.0, current_ma.0);
              }
        },
        Event::BeginLevitationOperatorCommand=> {
              if self.levitation_systems_ready {
                  self.transition_to(State::BeginLevitation).await;
              } else {
                  warn!("Cannot start levitation, systems not ready");
              }
          },
        _ => { 
            debug! ("Event {} is ignored in current state",event)
            }
    }
}


// --------- BEGIN LEVITATION --------- 

async fn entry_begin_levitation(&mut self) {
    info! ("Levitation started");
    EVENT_BUS.sender().send(Event::StartLevitationCommand).await;


}

async fn react_begin_levitation(&mut self, event: Event) {
    match event {
        Event::LevitationStarted { initial_current_ma, initial_airgap_mm, target_airgap_mm } => {
            info! ("Status: initial current: {}ma, initial airgap: {}mm, target airgap: {:?}mm",
            initial_current_ma.0, initial_airgap_mm.0, target_airgap_mm.0);
             EVENT_BUS.sender().send(Event::RetractLateralSuspensionCommand).await;
        },
        Event::LateralSuspensionRetracted { actuator_pressure_bar, timestamp_ms } => {
            info!("Lateral suspension retracted: pressure={}bar at {}ms",
            actuator_pressure_bar.0, timestamp_ms.0);
        },
        Event::LevitationStatus { current_airgap_mm, target_airgap_mm, current_ma } => {
            // calculate absolute distance 
            let dist_to_target = current_airgap_mm.distance_to(target_airgap_mm);
            info! ("Status: current: {:?}ma, distance to target airgap: {}mm",current_ma.0, dist_to_target);

            if dist_to_target < 5 {  // TODO later: 5 is a placeholder, check with levitation team for real number
                // TODO later: track how long we've been stable before transitioning to ensure its not a fluctuation
                info!("Levitation stable, transitioning to Ready");
                self.transition_to(State::Ready).await;
            }
        },
        Event::LevitationStopped { final_airgap_mm, final_current_ma } => {}, // TODO decide if we need this 
         _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- READY --------- 

async fn entry_ready(&mut self) {
    info! ("Pod is ready");
    info! ("Levitation is stable, awaiting accelerate command from operator")
}

async fn react_ready(&mut self, event: Event) {
    match event {
        // GO
        Event::AccelerateOperatorCommand=> {
            // TODO do we need to have been ready for a certain threshold of time?
            info! ("Starting acceleration");
            self.transition_to(State::Accelerate).await;
        },
        Event::LevitationStatus { current_airgap_mm, target_airgap_mm, current_ma } => {
            let dist_to_target = current_airgap_mm.distance_to(target_airgap_mm);

            // Handle if we drift too far from target
            if dist_to_target > 10 {  // TODO check with levitation team how big
                warn!("Levitation unstable: {}mm from target", dist_to_target);
            }
        },
        // TODO decide if we need this 
        Event::LevitationFailed { reason, current_airgap_mm, current_ma } => {
            warn!("Levitation failed: reason={}, airgap={}mm, current={}mA",
            reason, current_airgap_mm.0, current_ma.0);
            self.transition_to(State::Emergency).await;

        },

        // Abort
        Event::EmergencyStopOperatorCommand => {
            warn! ("EMERGENCY STOP PRESSED");
            self.transition_to(State::Emergency).await;
        },
        Event::StopLevitationCommand => {
            info! ("Stop levitation requested");
            self.transition_to(State::StopLevitation).await;
        },
         _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- ACCELERATE --------- 

async fn entry_accelerate(&mut self) {
    info! ("Pod is accelerating");
    EVENT_BUS.sender().send(Event::StartPropulsionAccelerationCommand).await;
}
async fn react_accelerate(&mut self, event: Event) {
    match event {
    Event::BrakeOperatorCommand => {
        info!("Operator initiated braking");
        self.transition_to(State::Brake).await;
    }
    Event::PropulsionAccelerationStarted { timestamp_ms } => {
        info!("Acceleration started at {}ms", timestamp_ms.0);
    },
    Event::PropulsionStatus { 
        current_ma, 
        velocity_kmh, 
        temperature_c,
        voltage_mv, 
        frequency_hz, 
        force_n,
        } => {
      info!("Propulsion status: {}mA, {}km/h, {}°C, {}mV, {}Hz, {}N",
          current_ma.0,
          velocity_kmh.0,
          temperature_c.0,
          voltage_mv.0,
          frequency_hz.0,
          force_n.0
      );
        },
        // TODO: need navigation logic here 
        // If reaching end of track, brake
        _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- BRAKE --------- 

async fn entry_brake(&mut self) {
    info! ("Pod is braking");
    EVENT_BUS.sender().send(Event::StartPropulsionBrakingCommand).await;
    EVENT_BUS.sender().send(Event::ClampBrakesCommand).await;
}

async fn react_brake(&mut self, event: Event) {
    match event {
    Event::PropulsionBrakingStarted { timestamp_ms } => {
        info!("Braking started at {}ms", timestamp_ms.0);
    },
    Event::BrakesClamped { actuator_pressure_bar, timestamp_ms } => {
              info!("Brakes clamped: pressure={}bar at {}ms",
                  actuator_pressure_bar.0, timestamp_ms.0);
          },
    Event::PropulsionStatus { 
        current_ma, 
        velocity_kmh, 
        temperature_c,
        voltage_mv, 
        frequency_hz, 
        force_n,
        } => {
            info!("Propulsion status: {}mA, {}km/h, {}°C, {}mV, {}Hz, {}N",
          current_ma.0,
          velocity_kmh.0,
          temperature_c.0,
          voltage_mv.0,
          frequency_hz.0,
          force_n.0
      );
      info!("Braking: velocity={}km/h", velocity_kmh.0);

      // Check if stopped
      if velocity_kmh.0 == 0 {
          info!("Pod has stopped, ready for stop levitation command");
          // TODO: auto transition or wait for operator?
      }
        },
    Event::StopLevitationOperatorCommand => {
        info! ("Stop levitation pressed");
        self.transition_to(State::StopLevitation).await;
    }
        _ => { 
            debug! ("Event {} is ignored in current state",event)
        }

    }
}


// --------- STOP LEVITATION --------- 

async fn entry_stop_levitation(&mut self) {
    info! ("Levitation stopping");
    info! ("Extending lateral suspension");
    EVENT_BUS.sender().send(Event::ExtendLateralSuspensionCommand).await;
}
async fn react_stop_levitation(&mut self, event: Event) {
    match event {
        Event::LateralSuspensionExtended { actuator_pressure_bar, timestamp_ms } => {
              info!("Lateral suspension extended: pressure={}bar at {}ms",
                  actuator_pressure_bar.0, timestamp_ms.0);
              EVENT_BUS.sender().send(Event::StopLevitationCommand).await;
          },
        Event::LevitationStopped { final_airgap_mm, final_current_ma } => {
            info!("Levitation stopped: airgap={}mm, current={}mA",
                  final_airgap_mm.0, final_current_ma.0);
              self.transition_to(State::Stopped).await;
        }
    _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- STOPPED --------- 

async fn entry_stopped(&mut self) {
    info! ("Pod is stopped");
    EVENT_BUS.sender().send(Event::StartDischargeCommand).await;
}
async fn react_stopped(&mut self, event: Event) {
    match event {
        Event::DischargeStarted { from, timestamp_ms } => {
            info! ("Board {:?} started discharge at {}ms",from, timestamp_ms.0);
        },
        Event::DischargeComplete { from, timestamp_ms, voltage_final_mv } => {
            info! ("Board {:?} completed discharge at {}ms",from, timestamp_ms.0);
            self.boards_discharged.insert(from);

            // TODO do we need to check specific boards?
            // TODO should it be == or >= here? 
            // Can use precharged, since its the same
            // ITODO check: is is only electronics that discharges, or also motor controller?
            if !self.desired_boards_to_charge.is_empty() && self.boards_discharged.len() == self.desired_boards_to_charge.len() {
                info! ("Necessary boards discharged");
                // TODO implement which boards must be discharged
                self.transition_to(State::Idle).await;
            }
        },
    _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }
}


// --------- EMERGENCY --------- 

async fn entry_emergency(&mut self) {
    warn!("EMERGENCY STATE ENTERED");
    EVENT_BUS.sender().send(Event::ClampBrakesCommand).await;
    EVENT_BUS.sender().send(Event::ExtendLateralSuspensionCommand).await;
    EVENT_BUS.sender().send(Event::StopLevitationCommand).await;
    EVENT_BUS.sender().send(Event::StartPropulsionBrakingCommand).await;
}

async fn react_emergency(&mut self, event: Event) {
    match event {
        Event::BrakesClamped { actuator_pressure_bar, timestamp_ms } => {
              info!("Emergency brakes engaged: pressure={}bar at {}ms",
                  actuator_pressure_bar.0, timestamp_ms.0);
          },

          Event::LateralSuspensionExtended { actuator_pressure_bar, timestamp_ms } => {
              info!("Emergency suspension extended: pressure={}bar at {}ms",
                  actuator_pressure_bar.0, timestamp_ms.0);
          },
    // TODO decide what to do here 
    _ => { 
            debug! ("Event {} is ignored in current state",event)
        }
    }   
}
}


#[cfg(test)]
mod tests {
    use super::*;
    use hyped_communications::events::{Airgap, Current, Timestamp, Voltage};

    #[test]
    fn state_machine_defaults() {
        let mut sm = StateMachine::new();
        assert!(matches!(sm.current_state, State::Idle));
        assert!(!sm.levitation_systems_ready);
        assert_eq!(sm.total_boards, 5);
        assert!(sm.boards_calibrated.is_empty());
    }
}