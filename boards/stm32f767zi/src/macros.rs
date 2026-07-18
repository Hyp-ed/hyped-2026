/// Sends an emergency message over CAN with the given reason.
/// Will cause all boards to transition to the Emergency state.
#[macro_export]
macro_rules! emergency {
    ($reason:expr) => {
        let can_sender = CAN_SEND.sender();
        let can_message = CanMessage::Emergency(THIS_BOARD.get().await.clone(), $reason);
        can_sender.send(can_message).await;
        let emergency_sender = EMERGENCY.sender();
        emergency_sender.send(true);
    };
}

/// Perform default CAN configuration.
#[macro_export]
macro_rules! default_can_config {
    ($can:ident) => {
        $can.modify_filters()
            .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
        $can.modify_config().set_bitrate(500_000);
    };
}
