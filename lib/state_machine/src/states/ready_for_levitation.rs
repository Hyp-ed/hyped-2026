// use crate::{state::State, state_machine::StateMachine};
// use embassy_time::Instant;
// use hyped_communications::{bus::EVENT_BUS, events::Event};
// use hyped_core::logging::{debug, info, warn};

// impl StateMachine {
//     pub(crate) async fn entry_ready_for_levitation(&mut self) {
//         info!("Pod is ready for levitation");
//         EVENT_BUS.sender().send(Event::UnclampBrakesCommand).await;
//     }

//     pub(crate) async fn react_ready_for_levitation(&mut self, event: Event) {
//         match event {
//             Event::BrakesUnclamped { from } => {
//                 info!(
//                     "Brakes unclamped: board={} at {}ms",
//                     from,
//                     Instant::now().as_millis(),
//                 );
//                 self.brakes_clamped = false;
//             }
//             Event::DynamicsStatus {
//                 from,
//                 actuator_pressure_bar,
//             } => {
//                 info!(
//                     "Dynamics Status: board={}, actuator pressure={}bar at {}ms",
//                     from,
//                     actuator_pressure_bar,
//                     Instant::now().as_millis(),
//                 )
//             }
//             Event::LevitationSystemsReady => {
//                 info!("Levitation systems ready, awaiting operator command");
//                 //self.levitation_systems_ready = true;
//             }
//             Event::EmergencyStopOperatorCommand => {
//                 warn!("EMERGENCY STOP PRESSED");
//                 self.transition_to(State::Emergency).await;
//             }
//             _ => {
//                 debug!("Event {} is ignored in current state", event)
//             }
//         }
//     }
// }
