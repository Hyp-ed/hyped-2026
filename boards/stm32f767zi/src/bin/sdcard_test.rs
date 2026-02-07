#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hyped_boards_stm32f767zi::{
    sdmmc::{logging::LogBufWriter, sdmmc_task, LOG_CHANNEL},
    send_log_with_sender,
};
use hyped_communications::{
    boards::Board, data::CanData, emergency::Reason, measurements::MeasurementReading,
    messages::CanMessage, state_transition::StateTransitionCommand,
};
use hyped_core::config::MeasurementId;
use hyped_state_machine::states::State;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    spawner.spawn(sdmmc_task()).unwrap();
    let sender = LOG_CHANNEL.sender();

    for _ in 0..30 {
        send_log_with_sender!(
            sender,
            "{:?}",
            CanMessage::MeasurementReading(MeasurementReading {
                reading: CanData::Emergency(Reason::TemperatureLowerLimitFailure),
                board: Board::StateMachineTester,
                measurement_id: MeasurementId::AccelerometerAvg,
            })
        );

        send_log_with_sender!(
            sender,
            "{:?}",
            CanMessage::StateTransitionCommand(StateTransitionCommand {
                from_board: Board::StateMachineTester,
                to_state: State::ReadyForLevitation,
            })
        );
    }

    loop {
        core::hint::spin_loop();
    }
}
