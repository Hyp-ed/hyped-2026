use embassy_executor::task;
use embassy_stm32::can::Id;

use crate::{enqueue_canopen, NAV_TO_MTC_CMD_ID_EXT, NAV_TO_MTC_POS_TARGET_ID_EXT, NAV_TO_MTC_SPEED_ACCEL_ID_EXT};
use crate::motor_control::navigation::{decode_position_target_mm, decode_speed_accel_mmps, NAV_KINEMATICS};
use mc_logic::NavKinematics;
use hyped_motors::can_open_processor::Messages;

/// Reads frames from CAN1(RX1) which is reserved for the navigation commands.
/// These commands are parsed and converted into CANOpen drive commands that are
/// transmitted to the EMSISO motor controller over CAN2.
#[task]
pub async fn motor_control_loop(mut can1_rx: embassy_stm32::can::Receiver<'_>) {
    defmt::info!("Motor control loop started (Option A: separate CAN IDs)");

    loop {
        let env = match can1_rx.read().await {
            Ok(e) => e,
            Err(_e) => continue,
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
                    0x01 => { defmt::info!("NAV->MTC: START_DRIVE"); enqueue_canopen(Messages::StartDrive).await; }
                    0x02 => { defmt::info!("NAV->MTC: SHUTDOWN");    enqueue_canopen(Messages::Shutdown).await; }
                    0x03 => { defmt::info!("NAV->MTC: QUICK_STOP");  enqueue_canopen(Messages::QuickStop).await; }
                    0x04 => { defmt::info!("NAV->MTC: SWITCH_ON");   enqueue_canopen(Messages::SwitchOn).await; }

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
