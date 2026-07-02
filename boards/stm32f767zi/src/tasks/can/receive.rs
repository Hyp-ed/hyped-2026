use embassy_stm32::can::{CanRx, Id};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use hyped_can::HypedCanFrame;
use hyped_communications::{
    bus::publish,
    can_id::CanId,
    events::Event,
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    message_identifier::{EventId, MessageIdentifier},
    messages::CanMessage,
};
use hyped_sensors::imd::ImdFrame;

use crate::board_state::{EMERGENCY, THIS_BOARD};

use defmt_rtt as _;
use panic_probe as _;

/// Stores heartbeat messages coming in from other boards that we need to respond to.
pub static INCOMING_HEARTBEATS: Channel<CriticalSectionRawMutex, Heartbeat, 10> = Channel::new();

pub static INCOMING_IMD_MSGS: Channel<CriticalSectionRawMutex, ImdFrame, 10> = Channel::new();

/// Stores measurement readings coming in from other boards.
pub static INCOMING_MEASUREMENTS: Channel<CriticalSectionRawMutex, MeasurementReading, 10> =
    Channel::new();

/// Task that receives CAN messages and puts them into a channel.
/// Currently only supports `StateTransitionCommand`, `StateTransitionRequest` and `Heartbeat` messages.
#[embassy_executor::task]
pub async fn can_receiver(mut rx: CanRx<'static>) {
    let emergency_sender = EMERGENCY.sender();
    let incoming_heartbeat_sender = INCOMING_HEARTBEATS.sender();

    loop {
        defmt::debug!("Waiting for CAN message");

        let envelope = rx.read().await;
        if envelope.is_err() {
            continue;
        }
        let envelope = envelope.unwrap();

        if let Id::Extended(id) = envelope.frame.id() {
            const DEFAULT_IMD_ID: u32 = 0x18ff01f4;
            if id.as_raw() == DEFAULT_IMD_ID {
                let _ = INCOMING_IMD_MSGS.try_send(ImdFrame::from_data(envelope.frame.data()));
                continue;
            }
        }

        let id = envelope.frame.id();
        let raw_id = match id {
            Id::Standard(id) => id.as_raw() as u32, // 11-bit ID
            Id::Extended(id) => id.as_raw(),        // 29-bit ID
        };
        let mut data = [0u8; 8];
        data.copy_from_slice(envelope.frame.data());
        let can_frame = HypedCanFrame::new(raw_id, data);

        let source: CanId = raw_id.into();
        if source.board == *THIS_BOARD.get().await && is_own_command_loopback(&source) {
            defmt::debug!("Ignoring loopback command from {:?}", source.board);
            continue;
        }

        let can_message: CanMessage = can_frame.into();
        defmt::debug!("Received CAN message: {:?}", can_message);

        match can_message {
            CanMessage::Heartbeat(heartbeat) => {
                defmt::debug!("Received heartbeat: {:?}", heartbeat);
                // Never block RX on heartbeats — the listener may not be running (bench setup).
                let _ = incoming_heartbeat_sender.try_send(heartbeat);
            }
            CanMessage::Emergency(board, reason) => {
                emergency_sender.send(true);
                defmt::error!("Emergency message from board {}: {}", board, reason);
            }
            CanMessage::MeasurementReading(measurement_reading) => {
                defmt::info!("Received measurement reading: {:?}", measurement_reading);
                let _ = INCOMING_MEASUREMENTS.try_send(measurement_reading);
            }

            // Electronics
            CanMessage::StartPrechargeCommand => {
                defmt::debug!("Start Precharge Command received");
                publish(Event::StartPrechargeCommand).await;
            }
            CanMessage::StartDischargeCommand => {
                defmt::debug!("Start Discharge Command received");
                publish(Event::StartDischargeCommand).await;
            }
            CanMessage::PrechargeStarted => {
                defmt::debug!("Precharge started");
                publish(Event::PrechargeStarted).await;
            }
            CanMessage::DischargeStarted => {
                defmt::debug!("Discharge started");
                publish(Event::DischargeStarted).await;
            }
            CanMessage::PrechargeComplete => {
                defmt::debug!("Precharge complete");
                publish(Event::PrechargeComplete).await;
            }
            CanMessage::DischargeComplete => {
                defmt::debug!("Discharge complete");
                publish(Event::DischargeComplete).await;
            }
            CanMessage::VoltageStatus { voltage } => {
                defmt::debug!("Voltage status: {}cV", voltage.0);
                publish(Event::VoltageStatus { voltage }).await;
            }
            CanMessage::PrechargeVoltageOK => {
                defmt::debug!("Precharge voltage OK");
                publish(Event::PrechargeVoltageOK).await;
            }
            CanMessage::DischargeVoltageOK => {
                defmt::debug!("Discharge voltage OK");
                publish(Event::DischargeVoltageOK).await;
            }

            // Relays
            CanMessage::ShutdownCircuitryRelayOpen => {
                defmt::debug!("Shutdown circuitry relay open");
                publish(Event::ShutdownCircuitryRelayOpen).await;
            }
            CanMessage::ShutdownCircuitryRelayClosed => {
                defmt::debug!("Shutdown circuitry relay closed");
                publish(Event::ShutdownCircuitryRelayClosed).await;
            }
            CanMessage::BatteryPrechargeRelayOpen => {
                defmt::debug!("Battery precharge relay open");
                publish(Event::BatteryPrechargeRelayOpen).await;
            }
            CanMessage::BatteryPrechargeRelayClosed => {
                defmt::debug!("Battery precharge relay closed");
                publish(Event::BatteryPrechargeRelayClosed).await;
            }
            CanMessage::MotorControllerRelayOpen => {
                defmt::debug!("Motor controller relay open");
                publish(Event::MotorControllerRelayOpen).await;
            }
            CanMessage::MotorControllerRelayClosed => {
                defmt::debug!("Motor controller relay closed");
                publish(Event::MotorControllerRelayClosed).await;
            }
            CanMessage::DischargeRelayOpen => {
                defmt::debug!("Discharge relay open");
                publish(Event::DischargeRelayOpen).await;
            }
            CanMessage::DischargeRelayClosed => {
                defmt::debug!("Discharge relay closed");
                publish(Event::DischargeRelayClosed).await;
            }

            // Motor Controller
            CanMessage::MotorControllerSetOperationalCommand => {
                defmt::debug!("Motor Controller Set Operational Command received");
                publish(Event::MotorControllerSetOperationalCommand).await;
            }
            CanMessage::MotorControllerSetupCommand => {
                defmt::debug!("Motor controller setup Command received");
                publish(Event::MotorControllerSetupCommand).await;
            }
            CanMessage::OpenPrechargeRelaysCommand => {
                defmt::debug!("Open precharge relays command received");
                publish(Event::OpenPrechargeRelaysCommand).await;
            }
            CanMessage::MotorControllerSetupComplete => {
                defmt::debug!("Motor controller setup complete");
                publish(Event::MotorControllerSetupComplete).await;
            }
            CanMessage::MotorControllerOperational => {
                defmt::debug!("Motor controller Operational");
                publish(Event::MotorControllerOperational).await;
            }

            // Navigation
            CanMessage::EndOfTrackBrake => {
                defmt::debug!("End of Track Brake Command Received");
                publish(Event::EndOfTrackBrakeCommand).await;
            }

            // Dynamics
            CanMessage::UnclampBrakesCommand => {
                defmt::debug!("Unclamp Brakes Command Received");
                publish(Event::UnclampBrakesCommand).await;
            }
            CanMessage::ClampBrakesCommand => {
                defmt::debug!("Clamp Brakes Command Received");
                publish(Event::ClampBrakesCommand).await;
            }
            CanMessage::RetractLateralSuspensionCommand => {
                defmt::debug!("Retract Lateral Suspension Command Received");
                publish(Event::RetractLateralSuspensionCommand).await;
            }
            CanMessage::ExtendLateralSuspensionCommand => {
                defmt::debug!("Extend Lateral Suspension Command Received");
                publish(Event::ExtendLateralSuspensionCommand).await;
            }
            CanMessage::BrakesClamped { from } => {
                defmt::debug!("Brakes clamped. Board={}", from);
                publish(Event::BrakesClamped { from }).await;
            }
            CanMessage::BrakesUnclamped { from } => {
                defmt::debug!("Brakes unclamped. Board={}", from);
                publish(Event::BrakesUnclamped { from }).await;
            }
            CanMessage::LateralSuspensionRetracted { from } => {
                defmt::debug!("Lateral Suspension Retracted. Board={}", from);
                publish(Event::LateralSuspensionRetracted { from }).await;
            }
            CanMessage::LateralSuspensionExtended { from } => {
                defmt::debug!("Lateral Suspension Extended. Board={}", from);
                publish(Event::LateralSuspensionExtended { from }).await;
            }
            CanMessage::DynamicsStatus {
                from,
                actuator_pressure_bar,
            } => {
                defmt::debug!(
                    "Dynamics Status: board={}, acutator pressure={}bar",
                    from,
                    actuator_pressure_bar
                );
                publish(Event::DynamicsStatus {
                    from,
                    actuator_pressure_bar,
                })
                .await;
            }

            // Propulsion
            CanMessage::StartPropulsionAccelerationCommand => {
                defmt::debug!("Start Propulsion Acceleration Command Received");
                publish(Event::StartPropulsionAccelerationCommand).await;
            }
            CanMessage::StartPropulsionBrakingCommand => {
                defmt::debug!("Start Propulsion Braking Command Received");
                publish(Event::StartPropulsionBrakingCommand).await;
            }
            CanMessage::PropulsionAccelerationStarted => {
                defmt::debug!("Propulsion Acceleration Started");
                publish(Event::PropulsionAccelerationStarted).await;
            }
            CanMessage::PropulsionBrakingStarted => {
                defmt::debug!("Propulsion Braking Started");
                publish(Event::PropulsionBrakingStarted).await;
            }
            CanMessage::PropulsionStatus {
                current_ma,
                velocity_kmh,
                temperature_c,
                voltage_cv,
            } => {
                defmt::debug!("Propulsion Status: current={}ma, velocity={}kmh, temperature={}c, voltage={}cv",
                current_ma.0, velocity_kmh.0, temperature_c.0, voltage_cv.0);
                publish(Event::PropulsionStatus {
                    current_ma,
                    velocity_kmh,
                    temperature_c,
                    voltage_cv,
                })
                .await;
            }
            CanMessage::PropulsionForce { force_n } => {
                defmt::debug!("Calculated propulsion force: force={}n", force_n.0);
                publish(Event::PropulsionForce { force_n }).await;
            }
        }
    }
}

/// Ignore loopback of commands this board transmitted. Status/completion frames must
/// still be accepted even if their CAN source board ID matches ours (legacy encoding).
fn is_own_command_loopback(source: &CanId) -> bool {
    matches!(
        source.message_identifier,
        MessageIdentifier::Event(
            EventId::StartPrechargeCommand
                | EventId::StartDischargeCommand
                | EventId::ClampBrakesCommand
                | EventId::UnclampBrakesCommand
                | EventId::StartPropulsionAccelerationCommand
                | EventId::StartPropulsionBrakingCommand
                | EventId::RetractLateralSuspensionCommand
                | EventId::ExtendLateralSuspensionCommand
                | EventId::EndOfTrackBrake
                | EventId::MotorControllerSetOperationalCommand
                | EventId::MotorControllerSetupCommand
                | EventId::OpenPrechargeRelaysCommand
        )
    )
}
