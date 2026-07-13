use crate::state::State;
use heapless::Vec as HeaplessVec;
use hyped_communications::{bus, bus::DynSubscriber, events::Event};
use hyped_core::logging::{debug, info, warn};

const MAX_PENDING_EVENTS: usize = 8;

#[derive(Debug, PartialEq, defmt::Format, Clone, Copy)]
pub enum PrechargeStep {
    Initial,
    ShutdownClosed,
    BatteryPrechargeClosed,
    AllClosed,
}

pub struct StateMachine {
    pub current_state: State,
    pub(crate) ready_for_run: bool,
    pub(crate) brakes_clamped: bool,
    pub(crate) shutdown_circuitry_relay_open: bool,
    pub(crate) precharge_voltage_ok: bool,
    pub(crate) precharge_complete: bool,
    pub(crate) discharge_voltage_ok: bool,
    pub(crate) discharge_complete: bool,
    pub(crate) motor_controller_setup_command_sent: bool,
    pub(crate) motor_controller_setup_done: bool,
    pub(crate) battery_precharge_relay_open: bool,
    pub(crate) motor_controller_relay_open: bool,
    pub(crate) motor_controller_operational_command_sent: bool,
    pub(crate) motor_controller_operational: bool,
    pub(crate) precharge_step: PrechargeStep,
    pending_events: HeaplessVec<Event, MAX_PENDING_EVENTS>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: State::Idle,
            ready_for_run: false,
            brakes_clamped: true,
            shutdown_circuitry_relay_open: false,
            precharge_step: PrechargeStep::Initial,
            motor_controller_setup_command_sent: false,
            motor_controller_setup_done: false,
            battery_precharge_relay_open: false,
            motor_controller_relay_open: false,
            motor_controller_operational_command_sent: false,
            motor_controller_operational: false,
            precharge_voltage_ok: false,
            precharge_complete: false,
            discharge_voltage_ok: false,
            discharge_complete: false,
            pending_events: HeaplessVec::new(),
        }
    }

    pub(crate) fn queue_publish(&mut self, event: Event) {
        if self.pending_events.push(event.clone()).is_err() {
            warn!("Pending event queue full, dropping {:?}", event);
        }
    }

    pub(crate) async fn drain_pending(&mut self) {
        while !self.pending_events.is_empty() {
            let event = self.pending_events.remove(0);
            bus::publish(event).await;
        }
    }

    fn publish_current_state(&mut self) {
        self.queue_publish(Event::StateChanged {
            state: self.current_state.telemetry_state(),
        });
    }

    fn publish_control_status(&mut self) {
        self.queue_publish(Event::ControlStatusChanged {
            can_setup_motor: self.current_state == State::Idle,
            can_precharge: self.current_state == State::SetupMotor
                && self.motor_controller_setup_done,
            can_ready_for_propulsion: self.current_state == State::Precharge && self.ready_for_run,
            can_accelerate: self.current_state == State::HvActive
                && self.precharge_step == PrechargeStep::AllClosed,
        });
    }

    // Actions when transitioning state
    pub(crate) async fn transition_to(&mut self, new_state: State) {
        if self.current_state == State::Emergency && new_state == State::Emergency {
            return;
        }
        info!("Transitioning: {:?} -> {:?}", self.current_state, new_state);
        self.current_state = new_state;
        self.publish_current_state();
        self.entry().await;
    }

    // Entry, match on state
    pub async fn entry(&mut self) {
        info!("Entering State: {:?}", self.current_state);

        match self.current_state {
            State::Idle => self.entry_idle().await,
            State::EnteringMaintenance => self.entry_entering_maintenance().await,
            State::Maintenance => self.entry_maintenance().await,
            State::SetupMotor => self.entry_setup_motor().await,
            State::Precharge => self.entry_precharge().await,
            State::HvActive => self.entry_hv_active().await,
            State::ReadyForPropulsion => self.entry_ready_for_propulsion().await,
            State::Accelerate => self.entry_accelerate().await,
            State::Brake => self.entry_brake().await,
            State::Stopped => self.entry_stopped().await,
            State::Emergency => self.entry_emergency().await,
        }
    }

    pub async fn react(&mut self, event: Event) {
        info!("React: {:?} in state {:?}", event, self.current_state);

        match event {
            // Emergency
            Event::Emergency { from, reason } => {
                warn!("EMERGENCY: from {:?} reason={}", from, reason);
                self.transition_to(State::Emergency).await;
                return;
            }
            Event::EmergencyStopOperatorCommand => {
                warn!("EMERGENCY: Operator sent emergency stop command");
                self.transition_to(State::Emergency).await;
                return;
            }
            Event::IdleOperatorCommand => {
                if self.current_state == State::Emergency {
                    warn!("Emergency is latched; use the explicit reset command");
                } else if matches!(self.current_state, State::Idle | State::Maintenance) {
                    warn!("Operator requested idle transition");
                    self.transition_to(State::Idle).await;
                } else if self.current_state == State::Stopped {
                    self.react_stopped(event).await;
                } else {
                    warn!("Idle can only be selected from Maintenance or Stopped");
                }
                return;
            }
            Event::MaintenanceOperatorCommand => {
                if self.current_state == State::Idle {
                    self.transition_to(State::EnteringMaintenance).await;
                } else {
                    warn!("Maintenance can only be entered from Idle");
                }
                return;
            }

            // Global events
            Event::Heartbeat { from } => {
                debug!("Heartbeat from {:?}", from);
            }

            _ => {}
        }

        if self.brake_signal_is_invalid(&event) {
            warn!("Emergency braking signal is invalid for the current state");
            self.transition_to(State::Emergency).await;
            return;
        }

        match self.current_state {
            State::Idle => self.react_idle(event).await,
            State::EnteringMaintenance => self.react_entering_maintenance(event).await,
            State::Maintenance => self.react_maintenance(event).await,
            State::SetupMotor => self.react_setup_motor(event).await,
            State::Precharge => self.react_precharge(event).await,
            State::HvActive => self.react_hv_active(event).await,
            State::ReadyForPropulsion => self.react_ready_for_propulsion(event).await,
            State::Accelerate => self.react_accelerate(event).await,
            State::Brake => self.react_brake(event).await,
            State::Stopped => self.react_stopped(event).await,
            State::Emergency => self.react_emergency(event).await,
        }
    }

    fn brake_signal_is_invalid(&self, event: &Event) -> bool {
        match event {
            Event::BrakesClamped { from } => {
                *from != hyped_communications::boards::Board::Pneumatics
                    || matches!(self.current_state, State::Maintenance | State::Accelerate)
            }
            Event::BrakesUnclamped { from } => {
                *from != hyped_communications::boards::Board::Pneumatics
                    || matches!(
                        self.current_state,
                        State::Idle
                            | State::SetupMotor
                            | State::Precharge
                            | State::HvActive
                            | State::Brake
                            | State::Stopped
                            | State::Emergency
                    )
            }
            _ => false,
        }
    }
}

#[embassy_executor::task]
pub async fn run(mut sm: StateMachine, mut events: DynSubscriber<'static, Event>) -> ! {
    sm.publish_current_state();
    sm.entry().await;
    sm.publish_control_status();
    sm.drain_pending().await;

    loop {
        let event = events.next_message_pure().await;
        if !matches!(
            event,
            Event::StateChanged { .. } | Event::ControlStatusChanged { .. }
        ) {
            sm.react(event).await;
        }

        sm.publish_control_status();
        sm.drain_pending().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicU64, Ordering},
        task::{Context, Poll},
    };
    use hyped_communications::boards::Board;
    use std::{
        boxed::Box,
        sync::Arc,
        task::{Wake, Waker},
    };

    struct NoopWaker;

    static TEST_TIME_TICKS: AtomicU64 = AtomicU64::new(0);

    #[no_mangle]
    fn _embassy_time_now() -> u64 {
        TEST_TIME_TICKS.fetch_add(1, Ordering::Relaxed)
    }

    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        let waker = Waker::from(Arc::new(NoopWaker));
        let mut context = Context::from_waker(&waker);
        let mut future: Pin<Box<F>> = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    fn has_pending(sm: &StateMachine, predicate: impl Fn(&Event) -> bool) -> bool {
        sm.pending_events.iter().any(predicate)
    }

    #[test]
    fn state_machine_defaults() {
        let sm = StateMachine::new();
        assert!(matches!(sm.current_state, State::Idle));
        assert!(!sm.ready_for_run);
        assert!(sm.brakes_clamped); // brakes start clamped
        assert!(!sm.shutdown_circuitry_relay_open);
        assert!(!sm.precharge_voltage_ok);
        assert!(!sm.precharge_complete);
        assert!(!sm.discharge_voltage_ok);
        assert!(!sm.discharge_complete);
        assert_eq!(sm.precharge_step, PrechargeStep::Initial);
    }

    #[test]
    fn maintenance_entry_isolates_hv_and_retracts_brakes() {
        let mut sm = StateMachine::new();

        block_on(sm.react(Event::MaintenanceOperatorCommand));

        assert_eq!(sm.current_state, State::EnteringMaintenance);
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::OpenPrechargeRelaysCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::UnclampBrakesCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionBrakingCommand
        )));

        block_on(sm.react(Event::ShutdownCircuitryRelayOpen));
        block_on(sm.react(Event::BatteryPrechargeRelayOpen));
        block_on(sm.react(Event::MotorControllerRelayOpen));
        block_on(sm.react(Event::BrakesUnclamped {
            from: Board::Pneumatics,
        }));

        assert_eq!(sm.current_state, State::Maintenance);
    }

    #[test]
    fn precharge_confirmation_enters_hv_active() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Precharge;
        sm.ready_for_run = true;
        sm.precharge_step = PrechargeStep::AllClosed;

        block_on(sm.react(Event::ReadyForPropulsionOperatorCommand));

        assert_eq!(sm.current_state, State::HvActive);
    }

    #[test]
    fn hv_active_keeps_propulsion_stopped_until_demo_is_requested() {
        let mut sm = StateMachine::new();
        sm.current_state = State::HvActive;
        sm.precharge_step = PrechargeStep::AllClosed;

        block_on(sm.entry());

        assert_eq!(sm.current_state, State::HvActive);
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionBrakingCommand
        )));
        assert!(!has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionAccelerationCommand
        )));

        block_on(sm.react(Event::AccelerateOperatorCommand));
        assert_eq!(sm.current_state, State::ReadyForPropulsion);
    }

    #[test]
    fn brake_deployment_during_demo_forces_emergency() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Accelerate;

        block_on(sm.react(Event::BrakesClamped {
            from: Board::Pneumatics,
        }));

        assert_eq!(sm.current_state, State::Emergency);
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::OpenPrechargeRelaysCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionBrakingCommand
        )));
    }

    #[test]
    fn commanded_demo_braking_uses_normal_brake_state() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Accelerate;

        block_on(sm.react(Event::BrakeOperatorCommand));

        assert_eq!(sm.current_state, State::Brake);
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::OpenPrechargeRelaysCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionBrakingCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::ClampBrakesCommand
        )));
    }

    #[test]
    fn stopped_requires_confirmed_discharge_and_explicit_idle_command() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Stopped;
        block_on(sm.entry());

        block_on(sm.react(Event::DischargeVoltageOK));
        block_on(sm.react(Event::DischargeComplete));
        assert_eq!(sm.current_state, State::Stopped);

        block_on(sm.react(Event::IdleOperatorCommand));
        assert_eq!(sm.current_state, State::Idle);
    }

    #[test]
    fn emergency_is_latched_against_idle_command() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Emergency;

        block_on(sm.react(Event::IdleOperatorCommand));

        assert_eq!(sm.current_state, State::Emergency);
    }

    #[test]
    fn idle_command_cannot_bypass_demo_safety_actions() {
        let mut sm = StateMachine::new();
        sm.current_state = State::Accelerate;

        block_on(sm.react(Event::IdleOperatorCommand));

        assert_eq!(sm.current_state, State::Accelerate);
    }

    #[test]
    fn emergency_entry_requests_all_safe_actions() {
        let mut sm = StateMachine::new();

        block_on(sm.transition_to(State::Emergency));

        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::OpenPrechargeRelaysCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::ClampBrakesCommand
        )));
        assert!(has_pending(&sm, |event| matches!(
            event,
            Event::StartPropulsionBrakingCommand
        )));
    }

    #[test]
    fn critical_brake_signal_from_wrong_board_forces_emergency() {
        let mut sm = StateMachine::new();

        block_on(sm.react(Event::BrakesClamped {
            from: Board::Navigation,
        }));

        assert_eq!(sm.current_state, State::Emergency);
    }
}
