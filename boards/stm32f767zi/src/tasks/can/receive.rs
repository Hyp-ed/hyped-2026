use embassy_stm32::can::{CanRx, Id};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use hyped_can::HypedCanFrame;
use hyped_communications::{
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    messages::CanMessage,
    bus::EVENT_BUS,
    events::Event};


use crate::board_state::EMERGENCY;

use defmt_rtt as _;
use panic_probe as _;

/// Stores heartbeat messages coming in from other boards that we need to respond to.
pub static INCOMING_HEARTBEATS: Channel<CriticalSectionRawMutex, Heartbeat, 10> = Channel::new();

/// Stores measurement readings coming in from other boards.
pub static INCOMING_MEASUREMENTS: Channel<CriticalSectionRawMutex, MeasurementReading, 10> =
    Channel::new();

/// Task that receives CAN messages and puts them into a channel.
/// Currently only supports `StateTransitionCommand`, `StateTransitionRequest` and `Heartbeat` messages.
#[embassy_executor::task]
pub async fn can_receiver(mut rx: CanRx<'static>) {
    let emergency_sender = EMERGENCY.sender();
    let incoming_heartbeat_sender = INCOMING_HEARTBEATS.sender();
    let event_sender = EVENT_BUS.sender();

    loop {
        defmt::debug!("Waiting for CAN message");

        let envelope = rx.read().await;
        if envelope.is_err() {
            continue;
        }
        let envelope = envelope.unwrap();
        let id = envelope.frame.id();
        let can_id = match id {
            Id::Standard(id) => id.as_raw() as u32, // 11-bit ID
            Id::Extended(id) => id.as_raw(),        // 29-bit ID
        };
        let mut data = [0u8; 8];
        data.copy_from_slice(envelope.frame.data());
        let can_frame = HypedCanFrame::new(can_id, data);

        let can_message: CanMessage = can_frame.into();
        defmt::debug!("Received CAN message: {:?}", can_message);

        match can_message {
            CanMessage::Heartbeat(heartbeat) => {
                defmt::debug!("Received heartbeat: {:?}", heartbeat);
                incoming_heartbeat_sender.send(heartbeat).await;
            }
            CanMessage::Emergency(board, reason) => {
                emergency_sender.send(true);
                defmt::error!("Emergency message from board {}: {}", board, reason);
            }
            CanMessage::MeasurementReading(measurement_reading) => {
                defmt::info!("Received measurement reading: {:?}", measurement_reading);
                INCOMING_MEASUREMENTS.send(measurement_reading).await;
            }
             
            // Calibration
            CanMessage::StartCalibrationCommand => {
                defmt::debug!("Received Start Calibration Command");
                event_sender.send(Event::StartCalibrationCommand).await;
            }
            CanMessage::CalibrationComplete { from }=> {
                defmt::debug!("Calibration completed on board={}",from);
                event_sender.send(Event::CalibrationComplete{ from }).await;
            }

            // Electronics
            CanMessage::StartPrechargeCommand => {
                defmt::debug!("Start Precharge Command received");
                event_sender.send(Event::StartPrechargeCommand).await;
            }
            CanMessage::StartDischargeCommand => {
                defmt::debug!("Start Discharge Command received");
                event_sender.send(Event::StartDischargeCommand).await;
            }
            CanMessage::PrechargeStarted { from } => {
                defmt::debug!("Precharge started on board={}",from);
                event_sender.send(Event::PrechargeStarted{ from }).await;
            }
            CanMessage::DischargeStarted { from } => {
                defmt::debug!("Discharge started on board={}",from);
                event_sender.send(Event::DischargeStarted{ from }).await;
            }
            CanMessage::PrechargeComplete {from, voltage} => {
                defmt::debug!("Board={} has completed precharge. Final voltage={}cv",from,voltage.0);
                event_sender.send(Event::PrechargeComplete{ from, voltage_cv: voltage }).await;
            }
            CanMessage::DischargeComplete {from, voltage} => {
                defmt::debug!("Board={} has completed discharge. Final voltage={}cv",from,voltage.0);
                event_sender.send(Event::DischargeComplete{ from, voltage_cv: voltage }).await;
            }
            CanMessage::PrechargeFailed {from, reason} => {
                defmt::debug!("Board={} has failed to precharge due to {}",from,reason);
                event_sender.send(Event::PrechargeFailed{ from, reason }).await;
            }


            // Levitation
            CanMessage::LevitationSystemsReady => {
                defmt::debug!("Levitation Systems Ready");
                event_sender.send(Event::LevitationSystemsReady).await;
            }
            CanMessage::StartLevitationCommand => {
                defmt::debug!("Start Levitation Command Received");
                event_sender.send(Event::StartLevitationCommand).await;
            }
            CanMessage::StopLevitationCommand => {
                defmt::debug!("Stop Levitation Command Received");
                event_sender.send(Event::StopLevitationCommand).await;
            }
            CanMessage::LevitationStarted {from} => {
                defmt::debug!("Levitation started on board {}",from);
                event_sender.send(Event::LevitationStarted{ from }).await;
            }
            CanMessage::LevitationStopped {from} => {
                defmt::debug!("Levitation stopped on board {}",from);
                event_sender.send(Event::LevitationStopped{ from }).await;
            }
            CanMessage::LevitationStatus {from, airgap_μm, current_ma} => {
                defmt::debug!("Levitation Status: board={}, airgap={}μm, current={}ma",from, airgap_μm.0, current_ma.0);
                event_sender.send(Event::LevitationStatus{ from, airgap_μm, current_ma }).await;
            }
            CanMessage::LevitationFailed {from, reason} => {
                defmt::debug!("Board={} has failed to levitate due to {}",from,reason);
                event_sender.send(Event::LevitationFailed{ from, reason }).await;
            }
            
            // Dynamics
            CanMessage::UnclampBrakesCommand => {
                defmt::debug!("Unclamp Brakes Command Received");
                event_sender.send(Event::UnclampBrakesCommand).await;
            }
            CanMessage::ClampBrakesCommand => {
                defmt::debug!("Clamp Brakes Command Received");
                event_sender.send(Event::ClampBrakesCommand).await;
            }
            CanMessage::RetractLateralSuspensionCommand => {
                defmt::debug!("Retract Lateral Suspension Command Received");
                event_sender.send(Event::RetractLateralSuspensionCommand).await;
            }
            CanMessage::ExtendLateralSuspensionCommand => {
                defmt::debug!("Extend Lateral Suspension Command Received");
                event_sender.send(Event::ExtendLateralSuspensionCommand).await;
            }
            CanMessage::BrakesClamped {from} => {
                defmt::debug!("Brakes clamped. Board={}",from);
                event_sender.send(Event::BrakesClamped{ from }).await;
            }
            CanMessage::BrakesUnclamped {from} => {
                defmt::debug!("Brakes unclamped. Board={}",from);
                event_sender.send(Event::BrakesUnclamped{ from }).await;
            }
            CanMessage::LateralSuspensionRetracted {from} => {
                defmt::debug!("Lateral Suspension Retracted. Board={}",from);
                event_sender.send(Event::LateralSuspensionRetracted{ from }).await;
            }
            CanMessage::LateralSuspensionExtended{from} => {
                defmt::debug!("Lateral Suspension Extended. Board={}",from);
                event_sender.send(Event::LateralSuspensionExtended{ from }).await;
            }
            CanMessage::DynamicsStatus{from, actuator_pressure_bar} => {
                defmt::debug!("Dynamics Status: board={}, acutator pressure={}bar",from,actuator_pressure_bar);
                event_sender.send(Event::DynamicsStatus{from, actuator_pressure_bar}).await;
            }

            // Propulsion
            CanMessage::StartPropulsionAccelerationCommand => {
                defmt::debug!("Start Propulsion Acceleration Command Received");
                event_sender.send(Event::StartPropulsionAccelerationCommand).await;
            }
            CanMessage::StartPropulsionBrakingCommand => {
                defmt::debug!("Start Propulsion Braking Command Received");
                event_sender.send(Event::StartPropulsionBrakingCommand).await;
            }
            CanMessage::PropulsionAccelerationStarted => {
                defmt::debug!("Propulsion Acceleration Started");
                event_sender.send(Event::PropulsionAccelerationStarted).await;
            }
            CanMessage::PropulsionBrakingStarted => {
                defmt::debug!("Propulsion Braking Started");
                event_sender.send(Event::PropulsionBrakingStarted).await;
            }
            CanMessage::PropulsionStatus{ current_ma, velocity_kmh, temperature_c, voltage_cv} => {
                defmt::debug!("Propulsion Status: current={}ma, velocity={}kmh, temperature={}c, voltage={}cv",
                current_ma.0, velocity_kmh.0, temperature_c.0, voltage_cv.0);
                event_sender.send(Event::PropulsionStatus{
                    current_ma, 
                    velocity_kmh,
                    temperature_c,
                    voltage_cv}).await;
            }
            CanMessage::PropulsionForce{ force_n } => {
                defmt::debug!("Calculated propulsion force: force={}n",force_n.0);
                event_sender.send(Event::PropulsionForce{ force_n }).await;
            }
            CanMessage::PropulsionFailed{from, reason} => {
                defmt::debug!("Propulsion failed: board={}, reason={}",from,reason);
                event_sender.send(Event::PropulsionFailed{ from, reason }).await;
            }
        }
    }
}
