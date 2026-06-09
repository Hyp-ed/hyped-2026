use crate::events::Event;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};

pub use embassy_sync::pubsub::DynSubscriber;

pub const EVENT_BUS_CAPACITY: usize = 64;
pub const EVENT_BUS_SUBSCRIBERS: usize = 2;
pub const EVENT_BUS_PUBLISHERS: usize = 1;

// Broadcast event bus. Every subscriber receives a copy of each published event.
pub static EVENT_BUS: PubSubChannel<
    CriticalSectionRawMutex,
    Event,
    EVENT_BUS_CAPACITY,
    EVENT_BUS_SUBSCRIBERS,
    EVENT_BUS_PUBLISHERS,
> = PubSubChannel::new();

// Initialise the event bus. Subscribers must be created before any events are published.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum Error {
    LimitReached,
}

pub fn init() -> Result<(), Error> {
    Ok(())
}

// Create a subscriber. Must be called before any events are published.
pub fn subscriber() -> Result<DynSubscriber<'static, Event>, Error> {
    EVENT_BUS.dyn_subscriber().map_err(|_| Error::LimitReached)
}

// Publish an event to all subscribers without blocking.
pub async fn publish(event: Event) {
    EVENT_BUS.immediate_publisher().publish_immediate(event);
}
