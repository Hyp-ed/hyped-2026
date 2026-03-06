#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    mode::Async,
    usart::{Config, InterruptHandler, Uart},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, watch::Watch};
use embassy_time::Duration;
use hyped_boards_stm32f767zi::{
    board_state::THIS_BOARD, io::Stm32f767ziUart, tasks::sensors::read_lp_bms_uart,
};
use hyped_communications::boards::Board;
use hyped_core::config::MeasurementId;
use hyped_sensors::{lp_bms::BatteryData, lp_bms_uart::BmsUart};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USART3 => InterruptHandler<embassy_stm32::peripherals::USART3>;
});

/// Used to keep the latest BMS data.
pub static CURRENT_BMS_DATA: Watch<CriticalSectionRawMutex, Option<BatteryData>, 1> = Watch::new();
static UART_MUTEX: StaticCell<Mutex<CriticalSectionRawMutex, Uart<'static, Async>>> =
    StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD
        .init(Board::Test)
        .expect("Failed to initialize board");

    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let uart: Uart<'static, Async> = Uart::new(
        p.USART3,   // peripheral (USART1–USART6 available)
        p.PD9,      // RX pin
        p.PD8,      // TX pin
        Irqs,       // interrupt binding
        p.DMA1_CH3, // TX DMA channel
        p.DMA1_CH1, // RX DMA channel
        config,
    )
    .expect("Failed to initialize uart");

    let uart_mutex = Mutex::new(uart);
    let reference = UART_MUTEX.init(uart_mutex);

    let uart = Stm32f767ziUart::new(reference);

    // Create a sender to pass to the BMS reading task, and a receiver for reading the values back.
    let mut receiver = CURRENT_BMS_DATA.receiver().unwrap();

    // spawner.must_spawn(state_updater());
    // spawner.must_spawn(heartbeat_listener(Board::Telemetry));
    // spawner.must_spawn(send_heartbeat(Board::Telemetry));

    let mut bms = BmsUart::new(uart);

    bms.init().await.expect("Failed to init");

    spawner.must_spawn(read_lp_bms_uart::read_lp_bms(
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
