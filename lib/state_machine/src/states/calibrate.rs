// use crate::{state::State, state_machine::StateMachine};
// use hyped_communications::{bus::EVENT_BUS, events::Event};
// use hyped_core::logging::{debug, info};

// impl StateMachine {
//     pub(crate) async fn entry_calibrate(&mut self) {
//         info!("Starting calibration");
//         // Reset tracking
//         self.boards_calibrated.clear();

//         // Tell boards to start calibration
//         EVENT_BUS
//             .sender()
//             .send(Event::StartCalibrationCommand)
//             .await;
//     }

//     pub(crate) async fn react_calibrate(&mut self, event: Event) {
//         match event {
//             Event::CalibrationComplete { from } => {
//                 info!("Board {:?} calibrated", from);
//                 // Track which boards are calibrated
//                 let _ = self.boards_calibrated.insert(from);

//                 // Check if all are done
//                 if self.boards_calibrated.len() >= self.total_boards as usize {
//                     info!("All boards calibrated");
//                     self.transition_to(State::Precharge).await;
//                 }
//             }
//             _ => {
//                 debug!("Event {} is ignored in current state", event)
//             }
//         }
//     }
// }
