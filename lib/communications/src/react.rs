use defmt::info;

use crate::boards::Board;
use crate::bus::EVENT_BUS;
use crate::emergency::Reason;
use crate::events::{Event, Nature, StateEvent};

pub async fn react(nature: Nature, from: Option<Board>, emergency_reason: Option<u8>) {
	match nature {
        /// Will be handled differently after microstates are well defined 
        /// This is primitive logging for now
		Nature::MinorChange => {
			info!("[react] minor change event");
			let ev = Event::StateEvent(StateEvent::Minor);
			let _ = EVENT_BUS.sender().send(ev).await;
		}

		Nature::MajorChange => {
			info!("[react] major change event");
			let ev = Event::StateEvent(StateEvent::Major);
			let _ = EVENT_BUS.sender().send(ev).await;
		}

		Nature::DirEmergency => {
			let board = from.unwrap();
			let reason_code = emergency_reason.unwrap_or(Reason::Unknown as u8);
			let reason = Reason::try_from(reason_code)
				.expect("Invalid reason for emergency stop");

			info!("[react] direct emergency from {:?} (reason={:?})", board, reason);

			let ev = Event::Emergency {
				from: board,
				reason: reason_code,
			};

			let _ = EVENT_BUS.sender().send(ev).await;
		}
	}
}

