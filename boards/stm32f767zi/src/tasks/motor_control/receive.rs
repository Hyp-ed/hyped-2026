use defmt::*;
use embassy_stm32::can::CanRx;

#[embassy_executor::task]
pub async fn motor_rx_task(mut rx: CanRx<'static>) {
    loop {
        match rx.read().await {
            Ok(envelope) => {
                let frame = envelope.frame;
                let raw_id: u32 = match frame.id() {
                    embassy_stm32::can::Id::Standard(id) => id.as_raw() as u32,
                    embassy_stm32::can::Id::Extended(id) => id.as_raw(),
                };
                info!(
                    "Motor RX: id={=u32:x} data={=[u8]:#02x}",
                    raw_id,
                    frame.data()
                );
            }
            Err(e) => {
                warn!("Motor CAN receive error: {:?}", e);
                continue;
            }
        }
    }
}
