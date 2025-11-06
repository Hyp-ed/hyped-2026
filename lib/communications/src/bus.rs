use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use crate::events::Event;


pub static EVENT_BUS: Channel<CriticalSectionRawMutex, Event, 64> = Channel::new();