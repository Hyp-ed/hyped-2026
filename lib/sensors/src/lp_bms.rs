/// Driver for the TinyBMS s516 30A Battery Management System using CAN.
/// Used to monitor battery status and health.
/// For now it is assumed that the BMS communicates with another bus
use defmt::Format;
use embassy_stm32::can::{frame::Header, Frame, StandardId};
use embassy_sync_stm32::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Receiver, Sender},
};
use hyped_can::CanError;

/// Represents the parsed status from the BMS.
#[derive(Debug, PartialEq, Clone, Format)]
pub struct BatteryData {
    pub voltage: f32,
    pub current: f32,
    pub max_cell_mv: u16,
    pub min_cell_mv: u16,
    pub temperatures_c: [i16; 3], // internal, external1, external2
    pub cell_voltages_mv: heapless::Vec<u16, 32>,
}

pub struct Bms {
    can_rx: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 10>,
    can_tx: Sender<'static, CriticalSectionRawMutex, u8, 4>,
}

pub const BMS_NODE_ID: u8 = 0x01;
pub const BMS_REQUEST_ID: u32 = 0x400 | BMS_NODE_ID as u32;
pub const BMS_RESPONSE_ID: u32 = 0x500 | BMS_NODE_ID as u32;

impl Bms {
    pub fn new(
        can_rx: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 10>,
        can_tx: Sender<'static, CriticalSectionRawMutex, u8, 4>,
    ) -> Self {
        Self { can_rx, can_tx }
    }

    /// the can task will use the bms frame function to send it
    async fn send_simple_request(&mut self, cmd: u8) -> Result<(), CanError> {
        self.can_tx.send(cmd).await;

        Ok(())
    }

    async fn read_response(&mut self, expected_cmd: u8) -> Result<[u8; 8], CanError> {
        let data = loop {
            let data = self.can_rx.receive().await;
            if data[0] == expected_cmd {
                break data;
            }
        };

        Ok(data)
    }

    pub async fn read_voltage(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(0x14).await?;
        let data = self.read_response(0x14).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_current(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(0x15).await?;
        let data = self.read_response(0x15).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_max_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(0x16).await?;
        let data = self.read_response(0x16).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_min_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(0x17).await?;
        let data = self.read_response(0x17).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_temperatures(&mut self) -> Result<[i16; 3], CanError> {
        self.send_simple_request(0x1B).await?;
        let mut temps = [0i16; 3];
        for temp in &mut temps {
            let data = self.read_response(0x1B).await?;
            *temp = i16::from_le_bytes([data[2], data[3]]);
        }
        Ok(temps)
    }

    pub async fn read_cell_voltages(&mut self) -> Result<heapless::Vec<u16, 32>, CanError> {
        self.send_simple_request(0x1C).await?;
        let mut voltages = heapless::Vec::<u16, 32>::new();

        loop {
            let data = self.can_rx.receive().await;
            if data[1] == 0x1C {
                let val = u16::from_le_bytes([data[2], data[3]]);
                voltages.push(val).map_err(|_| CanError::BufferOverflow)?;
            } else {
                break;
            }
        }

        Ok(voltages)
    }

    pub async fn read_battery_data(&mut self) -> Result<BatteryData, CanError> {
        let voltage = self.read_voltage().await?;
        let current = self.read_current().await?;
        let max_cell_mv = self.read_max_cell_voltage().await?;
        let min_cell_mv = self.read_min_cell_voltage().await?;
        let temperatures_c = self.read_temperatures().await?;
        let cell_voltages_mv = self.read_cell_voltages().await?;

        Ok(BatteryData {
            voltage,
            current,
            max_cell_mv,
            min_cell_mv,
            temperatures_c,
            cell_voltages_mv,
        })
    }
}

pub fn bms_frame(cmd: u8) -> Option<Frame> {
    Frame::new(
        Header::new(
            embassy_stm32::can::Id::Standard(StandardId::new(BMS_REQUEST_ID as u16)?),
            0,
            false,
        ),
        &[cmd, 0, 0, 0, 0, 0, 0, 0],
    )
    .ok()
}
