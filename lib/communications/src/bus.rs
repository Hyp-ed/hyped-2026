use crate::events::Event;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub static EVENT_BUS: Channel<CriticalSectionRawMutex, Event, 64> = Channel::new();
