#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter::Mask32, Can, CanRx, CanTx, Fifo, Rx0InterruptHandler, Rx1InterruptHandler,
        SceInterruptHandler, TxInterruptHandler,
    },
    peripherals::CAN1,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;
use static_cell::StaticCell;

const NODE_ID: u16 = 0x01;
const SDO_TX_ID: u16 = 0x600 + NODE_ID;
const SDO_RX_ID: u16 = 0x580 + NODE_ID;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Setting up CAN1...");
    static CAN: StaticCell<Can<'static>> = StaticCell::new();
    let can = CAN.init(Can::new(p.CAN1, p.PD0, p.PD1, Irqs));

    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    can.modify_config().set_bitrate(500_000);
    can.enable().await;

    info!("CAN1 enabled");

    let (mut tx, mut rx) = can.split();

    Timer::after(Duration::from_millis(200)).await;

    info!("Reading warning object 0x2027:00...");
    send_sdo_read(&mut tx, 0x2027, 0x00).await;

    loop {
        let envelope = rx.read().await.unwrap();
        let frame = envelope.frame;

        let raw_id: u16 = match frame.id() {
            embassy_stm32::can::Id::Standard(id) => id.as_raw(),
            embassy_stm32::can::Id::Extended(_) => {
                info!("Ignoring extended frame");
                continue;
            }
        };

        let data = frame.data();

        info!("Received id={=u16:x} data={=[u8]:#02x}", raw_id, data);

        if raw_id != SDO_RX_ID {
            continue;
        }

        if data.len() < 8 {
            warn!("Short SDO frame");
            continue;
        }

        match data[0] {
            0x43 => {
                if data[1] == 0x27 && data[2] == 0x20 && data[3] == 0x00 {
                    let warnings = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                    info!("Warning register 0x2027 = {=u32}", warnings);
                    decode_warning_bits(warnings);
                    break;
                }
            }
            0x4B => {
                if data[1] == 0x27 && data[2] == 0x20 && data[3] == 0x00 {
                    let warnings = u16::from_le_bytes([data[4], data[5]]) as u32;
                    info!("Warning register 0x2027 = {=u32}", warnings);
                    decode_warning_bits(warnings);
                    break;
                }
            }
            0x4F => {
                if data[1] == 0x27 && data[2] == 0x20 && data[3] == 0x00 {
                    let warnings = data[4] as u32;
                    info!("Warning register 0x2027 = {=u32}", warnings);
                    decode_warning_bits(warnings);
                    break;
                }
            }
            0x80 => {
                let abort_code = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                warn!(
                    "SDO abort for {:x}:{:x}, code=0x{=u32:x}",
                    0x2027u16,
                    0x00u8,
                    abort_code
                );
                break;
            }
            _ => {}
        }
    }

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

async fn send_sdo_read(tx: &mut CanTx<'static>, index: u16, sub_index: u8) {
    let id = embassy_stm32::can::Id::Standard(unwrap!(
        embassy_stm32::can::StandardId::new(SDO_TX_ID)
    ));

    let data = [
        0x40,
        (index & 0x00FF) as u8,
        ((index >> 8) & 0x00FF) as u8,
        sub_index,
        0x00,
        0x00,
        0x00,
        0x00,
    ];

    let frame = embassy_stm32::can::Frame::new_data(id, &data).unwrap();
    tx.write(&frame).await;
}

fn decode_warning_bits(warnings: u32) {
    if warnings == 0 {
        info!("No active warnings");
        return;
    }

    warn!("Active warnings:");

    if warnings & (1 << 0) != 0 {
        warn!(" - Controller temperature exceeded");
    }
    if warnings & (1 << 1) != 0 {
        warn!(" - Motor temperature exceeded");
    }
    if warnings & (1 << 2) != 0 {
        warn!(" - DC link under voltage");
    }
    if warnings & (1 << 3) != 0 {
        warn!(" - DC link over voltage");
    }
    if warnings & (1 << 4) != 0 {
        warn!(" - DC link over current");
    }
    if warnings & (1 << 5) != 0 {
        warn!(" - Stall protection active");
    }
    if warnings & (1 << 6) != 0 {
        warn!(" - Max velocity exceeded");
    }
    if warnings & (1 << 7) != 0 {
        warn!(" - BMS proposed power");
    }
    if warnings & (1 << 8) != 0 {
        warn!(" - Capacitor temperature exceeded");
    }
    if warnings & (1 << 9) != 0 {
        warn!(" - I2T protection");
    }
    if warnings & (1 << 10) != 0 {
        warn!(" - Field weakening active");
    }
}