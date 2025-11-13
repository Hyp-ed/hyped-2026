#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler, Id},
    peripherals::{self, CAN1, CAN2},
    rng::{self, Rng},
    Config,
};
use embassy_time::{Duration, Timer};

use hyped_boards_stm32f767zi::{
    board_state::{CURRENT_STATE, EMERGENCY, THIS_BOARD},
    default_can_config,
    tasks::{
        can::{
            board_heartbeat::heartbeat_listener,        
            // NOTE: doesn't include CAN reciever, this is done locally
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
// payload format proposal:
//   data[0] command code: 0x01=StartDrive, 0x02=Shutdown, 0x03=QuickStop, 0x04=SwitchOn, 0x05=SetFrequency

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    THIS_BOARD.init(Board::MotorControl).expect("Failed to initialize board");

    let p = embassy_stm32::init(Config::default());

    defmt::info!("Setting up CAN1 (main bus, listen-only)...");
    let mut can1 = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    default_can_config!(can1);
    can1.enable().await;
    let (_can1_tx, can1_rx) = can1.split();        
    // don't insantiate a sender for CAN1 
    defmt::info!("CAN1 ready");

    defmt::info!("Setting up CAN2 (EMSISO)...");
    let mut can2 = Can::new(p.CAN2, p.PB12, p.PB13, Irqs); // need to double check GPIO pins 
    default_can_config!(can2);
    can2.enable().await;
    let (can2_tx, _can2_rx) = can2.split();

    w
    spawner.must_spawn(can_sender(can2_tx));
    defmt::info!("CAN2 ready");

    spawner.must_spawn(heartbeat_listener(Board::Telemetry)); // listen-only
    spawner.must_spawn(emergency_handler());
    spawner.must_spawn(state_machine());
    spawner.must_spawn(motor_control_loop(can1_rx));

    loop { Timer::after(Duration::from_secs(1)).await; }
}

#[embassy_executor::task]
async fn emergency_handler() {
    let current_state_sender = CURRENT_STATE.sender();

    loop {
        if EMERGENCY.receiver().unwrap().get().await {
            defmt::error!("Emergency signal received! Enqueuing QUICK_STOP and halting...");
            // Immediately push a Quick Stop to EMSISO via CAN2
            enqueue_canopen(Messages::QuickStop).await;

            current_state_sender.send(State::Emergency);
            Timer::after(Duration::from_millis(200)).await;
            panic!("Emergency stop triggered");
        }
    }
}

async fn enqueue_canopen(message: Messages) {
    let msg: CanOpenMessage = message.into();
    let frame: HypedCanFrame = msg.into();
    let can_msg: CanMessage = frame.into();
    CAN_SEND.send(can_msg).await;
}

// This is used instead of the global can_reciever as we're only taking commands
// from navigation 
#[embassy_executor::task]
async fn motor_control_loop(mut can1_rx: embassy_stm32::can::Receiver<'_>) {
    defmt::info!("Motor control loop started (listening on CAN1, sending via CAN2)");

    loop {
        // Uses the Embassy receiver directly (no shared can_receiver task for CAN1)
        let env = match can1_rx.read().await {
            Ok(e) => e,
            Err(_e) => continue,
        };

        let can_id = match env.frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        };
        let data = env.frame.data();

        if can_id != NAV_TO_MTC_CMD_ID_EXT || data.is_empty() { continue; }

        match data[0] {
            0x01 => { defmt::info!("NAV→MTC: START_DRIVE");   enqueue_canopen(Messages::StartDrive).await; }
            0x02 => { defmt::info!("NAV→MTC: SHUTDOWN");      enqueue_canopen(Messages::Shutdown).await; }
            0x03 => { defmt::info!("NAV→MTC: QUICK_STOP");    enqueue_canopen(Messages::QuickStop).await; }
            0x04 => { defmt::info!("NAV→MTC: SWITCH_ON");     enqueue_canopen(Messages::SwitchOn).await; }
            // implement rest of the functions (frequency)
            other => defmt::warn!("Unknown NAV→MTC command byte: {}", other),
        }
    }
}