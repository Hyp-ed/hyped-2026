#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler},
    peripherals::CAN1,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD,
    tasks::{
        can::{
            board_heartbeat::{heartbeat_listener, send_heartbeat},
            receive::{can_receiver, INCOMING_BMS_MESSAGES},
            send::{can_sender, BMS_SEND},
        },
        sensors::read_lp_bms::read_lp_bms,
        state_machine::state_updater,
    },
};
use hyped_communications::boards::Board;
use hyped_core::config::MeasurementId;
use hyped_sensors::lp_bms::{BatteryData, Bms};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

/// Used to keep the latest BMS data.
pub static CURRENT_BMS_DATA: Watch<CriticalSectionRawMutex, Option<BatteryData>, 1> = Watch::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD
        .init(Board::Test)
        .expect("Failed to initialize board");

    let p = embassy_stm32::init(Default::default());

    let (can_tx, can_rx) = Can::new(p.CAN1, p.PD0, p.PD1, Irqs).split();
    spawner.must_spawn(can_receiver(can_rx, None));
    spawner.must_spawn(can_sender(can_tx, None));

    // Create a sender to pass to the BMS reading task, and a receiver for reading the values back.
    let mut receiver = CURRENT_BMS_DATA.receiver().unwrap();

    // Construct the BMS driver using the CAN peripheral.
    let bms_receiver: embassy_sync::channel::Receiver<
        'static,
        CriticalSectionRawMutex,
        [u8; 8],
        10,
    > = INCOMING_BMS_MESSAGES.receiver();

    let bms_sender: embassy_sync::channel::Sender<'static, CriticalSectionRawMutex, [u8; 8], 4> =
        BMS_SEND.sender();

    // spawner.must_spawn(state_updater());
    // spawner.must_spawn(heartbeat_listener(Board::Telemetry));
    // spawner.must_spawn(send_heartbeat(Board::Telemetry));

    let mut bms = Bms::new(bms_receiver, bms_sender);
    bms.init().await.expect("Failed to init");

    spawner.must_spawn(read_lp_bms(
        bms,
        MeasurementId::LpBms,
        CURRENT_BMS_DATA.sender(),
    ));

    loop {
        // Only prints when the BMS data changes.
        let new_bms_data = receiver.changed();
        // according to the config there should be one every 5 ms, so if there is nothing after
        // 10ms there is a communication error
        let new_bms_data =
            embassy_time::with_timeout(Duration::from_millis(100), new_bms_data).await;

        match new_bms_data {
            Ok(Some(data)) => {
                defmt::info!(
                "New BMS data: voltage={}V, current={}A, max_cell_mv={}, min_cell_mv={}, temps={:?}, cell_voltages={:?}",
                data.voltage,
                data.current,
                data.max_cell_mv,
                data.min_cell_mv,
                data.temperatures_c,
                data.cell_voltages_mv
            );

                defmt::info!("Check faults: {:?}", data.check_faults());
            }
            Ok(None) => {
                defmt::warn!("No data");
            }
            Err(_) => {
                defmt::warn!("Temp or Cell Voltage communication failure: timeout");
                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }
}
