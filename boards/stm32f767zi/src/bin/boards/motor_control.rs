#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler},
    gpio::{Level, Output, OutputType, Speed},
    peripherals::{self, CAN1, CAN2},
    rng,
    Config,
};
use embassy_time::{Duration, Timer};
use core::sync::atomic::{AtomicBool, Ordering};

mod motor_control;
use motor_control::{
    braking::braking_system_loop,
    control_loop::motor_control_loop,
    navigation::NAV_KINEMATICS,
};

use hyped_boards_stm32f767zi::{
    board_state::{CURRENT_STATE, EMERGENCY, THIS_BOARD},
    default_can_config,
    tasks::{
        can::{
            board_heartbeat::send_heartbeat,      
            receive::can_receiver,
            send::{can_sender, CAN_SEND},                 
        },
        state_machine::state_machine,
    },
};

use hyped_communications::boards::Board;
use hyped_communications::messages::CanMessage;
use hyped_motors::can_open_processor::Messages;          
use hyped_motors::can_open_message::CanOpenMessage;      
use hyped_can::HypedCanFrame;                            
use hyped_state_machine::states::State;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RNG       => rng::InterruptHandler<peripherals::RNG>;
    CAN1_RX0  => Rx0InterruptHandler<CAN1>;
    CAN1_RX1  => Rx1InterruptHandler<CAN1>;
    CAN1_SCE  => SceInterruptHandler<CAN1>;
    CAN1_TX   => TxInterruptHandler<CAN1>;
    CAN2_RX0  => Rx0InterruptHandler<CAN2>;
    CAN2_RX1  => Rx1InterruptHandler<CAN2>;
    CAN2_SCE  => SceInterruptHandler<CAN2>;
    CAN2_TX   => TxInterruptHandler<CAN2>;
});

// NAV -> MotorControl command ID
const NAV_TO_MTC_CMD_ID_EXT: u32 = 0x18FF_0301; // agree this with Navigation (29-bit Extended ID)
// Additional NAV -> MTC IDs to remove command byte 
const NAV_TO_MTC_SPEED_ACCEL_ID_EXT: u32 = 0x18FF_0302; // 8 bytes: speed + accel
const NAV_TO_MTC_POS_TARGET_ID_EXT: u32  = 0x18FF_0303; // 8 bytes: position + target
// payload format proposal:4
//   data[0] command code: 0x01=StartDrive, 0x02=Shutdown, 0x03=QuickStop, 0x04=SwitchOn, 0x05=SetFrequency

// === Braking constants (placeholder values; replace with real pod params) ===
pub(crate) const POD_MASS_KG: f32 = 200.0;           // TODO: set actual pod mass
pub(crate) const MAX_BRAKE_FORCE_N: f32 = 5000.0;    // TODO: measured clamp braking force
pub(crate) const BRAKE_MARGIN_M: f32 = 5.0;          // TODO: safety margin distance for brake trigger

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub(crate) static BRAKE_SOLENOID: Mutex<CriticalSectionRawMutex, Option<Output<'static>>> = Mutex::new(None);
pub(crate) static FORCE_BRAKE: AtomicBool = AtomicBool::new(false);

pub(crate) async fn engage_brake_solenoid() {
    let mut guard = BRAKE_SOLENOID.lock().await;
    match guard.as_mut() {
        Some(pin) => pin.set_low(),
        None => defmt::warn!("Brake solenoid pin not initialised; cannot engage"),
    }
}

pub(crate) async fn release_brake_solenoid() {
    let mut guard = BRAKE_SOLENOID.lock().await;
    match guard.as_mut() {
        Some(pin) => pin.set_high(),
        None => defmt::warn!("Brake solenoid pin not initialised; cannot release"),
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD.init(Board::MotorControl).expect("Failed to initialize board");

    let p = embassy_stm32::init(Config::default());

    defmt::info!("Setting up CAN1 (main bus)...");
    let mut can1 = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can1);
    can1.enable().await;

    // TODO: replace PC13 with the actual brake solenoid GPIO pin once wiring is finalised.
    // NOTE: Assumes active-low solenoid: LOW = brakes engaged, HIGH = brakes released.
    // Confirm polarity with pneumatics wiring.
    let brake_solenoid_pin = Output::new(p.PC13, Level::High, OutputType::PushPull, Speed::High);
    {
        let mut guard = BRAKE_SOLENOID.lock().await;
        *guard = Some(brake_solenoid_pin);
    }


    // TODO: configure CAN filters so NAV frames are routed to FIFO1
    // Dedicated system receiver (heartbeats, emergency, state transitions)
    let can1_rx_system = can1.receiver_for_fifo(Fifo::Rx0);
    spawner.must_spawn(can_receiver(can1_rx_system));

    // Dedicated NAV receiver
    let can1_rx_nav = can1.receiver_for_fifo(Fifo::Rx1);
    spawner.must_spawn(motor_control_loop(can1_rx_nav));
    defmt::info!("CAN1 ready");

    defmt::info!("Setting up CAN2 (EMSISO)...");
    let mut can2 = Can::new(p.CAN2, p.PB12, p.PB13, Irqs); // need to double check GPIO pins 
    default_can_config!(can2);
    can2.enable().await;
    let (can2_tx, _can2_rx) = can2.split();

    spawner.must_spawn(can_sender(can2_tx));
    defmt::info!("CAN2 ready");

    spawner.must_spawn(send_heartbeat(Board::MotorControl));
    spawner.must_spawn(emergency_handler());
    spawner.must_spawn(state_machine());

    spawner.must_spawn(braking_system_loop(
        NAV_KINEMATICS.receiver(),
    ));

    loop { Timer::after(Duration::from_secs(1)).await; }
}

#[embassy_executor::task]
async fn emergency_handler() {
    let current_state_sender = CURRENT_STATE.sender();

    loop {
        if EMERGENCY.receiver().unwrap().get().await {
            defmt::error!("Emergency signal received! Enqueuing QUICK_STOP and halting...");
            // Immediately push a Quick Stop to EMSISO via CAN2
            FORCE_BRAKE.store(true, Ordering::SeqCst);
            enqueue_canopen(Messages::QuickStop).await;
            engage_brake_solenoid().await;

            current_state_sender.send(State::Emergency);
            Timer::after(Duration::from_millis(200)).await;
            panic!("Emergency stop triggered");
        }
    }
}

pub(crate) async fn enqueue_canopen(message: Messages) {
    let msg: CanOpenMessage = message.into();
    let frame: HypedCanFrame = msg.into();
    let can_msg: CanMessage = frame.into();
    CAN_SEND.send(can_msg).await;
}
