use embassy_executor::task;
use embassy_stm32::can::Id;
use embassy_time::{Duration, Timer};
use core::sync::atomic::Ordering;

use crate::bin::boards::motor_control::{
    enqueue_canopen, NAV_TO_MTC_CMD_ID_EXT, NAV_TO_MTC_POS_TARGET_ID_EXT,
    NAV_TO_MTC_SPEED_ACCEL_ID_EXT,
};
use crate::motor_control::navigation::{
    decode_position_target_mm, decode_speed_accel_mmps, NavKinematics, NAV_KINEMATICS,
};
use crate::board_state::EMERGENCY;
use hyped_motors::can_open_processor::Messages;

/// Reads frames from CAN1(RX1) which is reserved for the navigation commands.
/// These commands are parsed and converted into CANOpen drive commands that are
/// transmitted to the EMSISO motor controller over CAN2.
#[task]
pub async fn motor_control_loop(mut can1_rx: embassy_stm32::can::CanRx<'static>) {
    defmt::info!("Motor control loop started (Option A: separate CAN IDs)");

    //Used AtomicU32 for better compilation and speed:
    //CAN_ERROR_COUNT: counts number of errors have occurred, start at 0 and increment on each error
    //CAN_ERROR_THRESHOLD: constant defining max number of errors before triggering emergency
    use core::sync::atomic::AtomicU32;
    static CAN_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
    const CAN_ERROR_THRESHOLD: u32 = 10;
    loop {
        // Uses the Embassy receiver directly (no shared can_receiver task for CAN1)
        let env = match can1_rx.read().await {
            Ok(e) => e,
            Err(e) => {
                //Atomically increases error counter by 1 and retrieves previous value, set to Ordering::Relaxed for minimal memory
                let count = CAN_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
                //log error with new count value
                defmt::warn!("Error receiving CAN frame on CAN1: {:?}, count: {}", e, count + 1);
                //Check threshold, if 10+ errors --> call panic! to send emergency via emergency channel
                if count >= CAN_ERROR_THRESHOLD {
                    defmt::error!("Exceeded CAN error threshold on CAN1, communication failure");
                    EMERGENCY.sender().send(true);
                    panic!("Terminating due to CAN communication failure");
                }
                //Try to recover
                Timer::after(Duration::from_millis(100)).await;
                continue;
            },
        };

        let can_id = match env.frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        };

        let data = env.frame.data();

        match can_id {
            // ===== Commands / drive control =====
            NAV_TO_MTC_CMD_ID_EXT => {
                if data.is_empty() {
                    defmt::warn!("NAV->MTC CMD: empty payload");
                    continue;
                }

                match data[0] {
                    //log error if start_drive fails to send
                    0x01 => {
                        defmt::info!("NAV→MTC: START_DRIVE");
                        if let Err(_) = enqueue_canopen(Messages::StartDrive).await {
                            defmt::error!("Failed to send START_DRIVE message");
                        };
                    }
                    //log error if shutdown fails to send
                    0x02 => {
                        defmt::info!("NAV→MTC: SHUTDOWN");
                        if let Err(_) = enqueue_canopen(Messages::Shutdown).await {
                            defmt::error!("Failed to send SHUTDOWN message");
                            EMERGENCY.sender().send(true); // trigger emergency if shutdown fails
                            panic!("Terminating due to failed SHUTDOWN message");
                        }
                    }
                    //log error and trigger emergency if quick-stop fails to send
                    0x03 => {
                        defmt::info!("NAV→MTC: QUICK_STOP");
                        if let Err(_) = enqueue_canopen(Messages::QuickStop).await {
                            defmt::error!("Failed to send QUICK_STOP message");
                            EMERGENCY.sender().send(true); // trigger emergency if quick stop fails
                            panic!("Terminating due to failed QUICK_STOP message");
                        }
                    }
                    0x04 => {
                        defmt::info!("NAV→MTC: SWITCH_ON");
                        if let Err(_) = enqueue_canopen(Messages::SwitchOn).await{
                            defmt::error!("Failed to send SWITCH_ON message");
                        }
                    }

                    // SetFrequency still lives here, because it’s a command that carries a u32
                    0x05 => {
                        if data.len() < 5 {
                            defmt::warn!("NAV->MTC: SET_FREQUENCY missing payload");
                            continue;
                        }
                        let freq = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
                        defmt::info!("NAV->MTC: SET_FREQUENCY freq={} Hz", freq);
                        enqueue_canopen(Messages::SetFrequency(freq)).await;
                    }

                    other => defmt::warn!("NAV->MTC CMD: unknown command byte {}", other),
                }
            }

            // ===== Kinematics: speed + accel =====
            NAV_TO_MTC_SPEED_ACCEL_ID_EXT => {
                let Some((speed, accel)) = decode_speed_accel_mmps(data) else {
                    defmt::warn!("NAV->MTC SPEED/ACCEL: payload too short (len={})", data.len());
                    continue;
                };

                defmt::info!("NAV->MTC: speed={} m/s accel={} m/s²", speed, accel);

                NAV_KINEMATICS
                    .send(NavKinematics {
                        velocity_mps: Some(speed),
                        accel_mps2: Some(accel),
                        ..Default::default()
                    })
                    .await;
            }

            // ===== Kinematics: position + target =====
            NAV_TO_MTC_POS_TARGET_ID_EXT => {
                let Some((position, target)) = decode_position_target_mm(data) else {
                    defmt::warn!("NAV->MTC POS/TARGET: payload too short (len={})", data.len());
                    continue;
                };

                defmt::info!("NAV->MTC: position={} m target={} m", position, target);

                NAV_KINEMATICS
                    .send(NavKinematics {
                        position_m: Some(position),
                        target_position_m: Some(target),
                        ..Default::default()
                    })
                    .await;
            }

            _ => {
                // ignore other CAN IDs
            }
        }
    }
}
