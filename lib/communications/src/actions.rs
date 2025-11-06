use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use crate::boards::Board;
// actions that can directly take place and actions with permissions
use crate::events::{ActionWPerm, DirAction};

#[derive(Debug,Clone)]
pub enum Command {
    DirAction(DirAction),
    PermissionGranted(ActionWPerm),
    PermissionDenied(ActionWPerm),
}

pub static INCOMING_ACTIONS: Channel<CriticalSectionRawMutex, (Board, Command), 10> =
    Channel::new();

#[embassy_executor::task]
pub async fn command_handler() {
    let mut rx = INCOMING_ACTIONS.receiver();

    loop {
        let (origin, command) = rx.receive().await;
        defmt::info!("Command from {:?}: {:?}", origin, command);

        match command {
            // Command list
        }
    }
}