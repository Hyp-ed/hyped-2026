/// Driver for the TinyBMS s516 30A Battery Management System using CAN.
/// Used to monitor battery status and health.
/// It is connected to the main CAN bus
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

struct BmsCmd {}

impl BmsCmd {
    const RESET_OR_CLEAN_EVENT_OR_STATUS: u8 = 0x02;
    const RESET_BMS: u8 = 0x05;
    const READ_REGISTERS: u8 = 0x03;

    const READ_VOLTAGE: u8 = 0x14;
    const READ_CURRENT: u8 = 0x15;
    const READ_MAX_CELL_VOLTAGE: u8 = 0x16;
    const READ_MIN_CELL_VOLTAGE: u8 = 0x17;
    const READ_TEMPERATURES: u8 = 0x1B;
    const READ_CELL_VOLTAGES: u8 = 0x1C;
}

pub struct Bms {
    can_rx: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 10>,
    can_tx: Sender<'static, CriticalSectionRawMutex, [u8; 8], 4>,

    cells_count: u16,
}

pub const BMS_NODE_ID: u8 = 0x01;
pub const BMS_REQUEST_ID: u32 = 0x400 | BMS_NODE_ID as u32;
pub const BMS_RESPONSE_ID: u32 = 0x500 | BMS_NODE_ID as u32;

impl Bms {
    const SUCCESS: u8 = 1;
    const IS_ERR: usize = 0;
    const CMD: usize = 1;
    const ERROR: usize = 3;
    const BMS_TEMPERATURE_COUNT: usize = 3;
    const CELL_COUNT_REGISTER: u8 = 53;

    pub fn new(
        can_rx: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 10>,
        can_tx: Sender<'static, CriticalSectionRawMutex, [u8; 8], 4>,
    ) -> Self {
        Self {
            can_rx,
            can_tx,
            cells_count: 0,
        }
    }

    pub async fn init(&mut self) -> Result<(), CanError> {
        self.reset().await?;
        Ok(self.cells_count = self.read_cell_count().await?)
    }

    /// the can task will use the bms frame function to send it
    async fn send_simple_request(&mut self, cmd: u8) -> Result<(), CanError> {
        self.can_tx.send([cmd, 0, 0, 0, 0, 0, 0, 0]).await;

        Ok(())
    }

    async fn read_response(&mut self, expected_cmd: u8) -> Result<[u8; 8], CanError> {
        let data = loop {
            let data = self.can_rx.receive().await;
            if data[Self::CMD] == expected_cmd {
                if data[Self::IS_ERR] != Self::SUCCESS {
                    defmt::warn!("BMS task failed with error {:?}", data[Self::ERROR]);
                    return Err(CanError::Unknown);
                }

                break data;
            }
        };

        Ok(data)
    }

    pub async fn read_voltage(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(BmsCmd::READ_VOLTAGE).await?;
        let data = self.read_response(BmsCmd::READ_VOLTAGE).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_current(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(BmsCmd::READ_CURRENT).await?;
        let data = self.read_response(BmsCmd::READ_CURRENT).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_max_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(BmsCmd::READ_MAX_CELL_VOLTAGE)
            .await?;
        let data = self.read_response(BmsCmd::READ_MAX_CELL_VOLTAGE).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_min_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(BmsCmd::READ_MIN_CELL_VOLTAGE)
            .await?;
        let data = self.read_response(BmsCmd::READ_MIN_CELL_VOLTAGE).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_temperatures(&mut self) -> Result<[i16; 3], CanError> {
        self.send_simple_request(BmsCmd::READ_TEMPERATURES).await?;
        let mut temps = [0i16; Self::BMS_TEMPERATURE_COUNT];
        for temp in &mut temps {
            let data = self.read_response(BmsCmd::READ_TEMPERATURES).await?;
            *temp = i16::from_le_bytes([data[2], data[3]]);
        }
        Ok(temps)
    }

    pub async fn read_cell_voltages(&mut self) -> Result<heapless::Vec<u16, 32>, CanError> {
        self.send_simple_request(BmsCmd::READ_CELL_VOLTAGES).await?;
        let mut voltages = heapless::Vec::<u16, 32>::new();

        loop {
            if voltages.len() >= self.cells_count as usize {
                break;
            }

            let data = self.can_rx.receive().await;
            if data[1] == BmsCmd::READ_CELL_VOLTAGES {
                let val = u16::from_le_bytes([data[2], data[3]]);
                voltages.push(val).map_err(|_| CanError::BufferOverflow)?;
            } else {
                break;
            }
        }

        Ok(voltages)
    }

    pub async fn reset(&mut self) -> Result<(), CanError> {
        self.can_tx
            .send([
                BmsCmd::RESET_OR_CLEAN_EVENT_OR_STATUS,
                BmsCmd::RESET_BMS,
                0,
                0,
                0,
                0,
                0,
                0,
            ])
            .await;
        self.read_response(BmsCmd::RESET_OR_CLEAN_EVENT_OR_STATUS)
            .await
            .map(|_| ())
    }

    pub async fn read_cell_count(&mut self) -> Result<u16, CanError> {
        self.can_tx
            .send([
                BmsCmd::READ_REGISTERS,
                Self::CELL_COUNT_REGISTER,
                0,
                0,
                1,
                0,
                0,
                0,
            ])
            .await;

        let data = self.read_response(BmsCmd::READ_REGISTERS).await?;

        Ok(u16::from_be_bytes([data[3], data[4]]))
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

pub fn bms_frame(cmd: [u8; 8]) -> Option<Frame> {
    Frame::new(
        Header::new(
            embassy_stm32::can::Id::Standard(StandardId::new(BMS_REQUEST_ID as u16)?),
            0,
            false,
        ),
        &cmd,
    )
    .ok()
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum BmsFault {
    // Cell Voltage Faults
    CellVoltageOpenWire(usize), // Cell index
    CellVoltageShortToSupply(usize),
    CellVoltageShortToGnd(usize),
    CellVoltageOutOfRange(usize),
    CellVoltageCommunicationFailure,

    // Temperature Faults
    TempSensorOpenWire(usize),
    TempSensorShortToSupply(usize),
    TempSensorShortToGnd(usize),
    TempOutOfRange(usize),
    TempCommunicationFailure,
}

impl BatteryData {
    // unit: millivolts
    const CELL_VOLTAGE_ZERO: u16 = 0;
    const CELL_VOLTAGE_SHORT_TO_GND_MAX: u16 = 100;
    const CELL_VOLTAGE_MIN_SAFE: u16 = 2000;
    const CELL_VOLTAGE_MAX_SAFE: u16 = 4500;
    const CELL_VOLTAGE_SHORT_TO_SUPPLY: u16 = 5000;

    // unit: degrees Celsius
    const TEMP_SENSOR_DISCONNECTED: i16 = -32768;
    const TEMP_MIN_SAFE: i16 = -40;
    const TEMP_SHORT_TO_GND_MIN: i16 = -50;
    const TEMP_MAX_SAFE: i16 = 90;
    const TEMP_SHORT_TO_SUPPLY: i16 = 150;

    /// This function checks for faults based on the battery data
    /// NOTE: it does not check for the communication failures
    pub fn check_faults(&self) -> Option<BmsFault> {
        for (idx, &cell_voltage) in self.cell_voltages_mv.iter().enumerate() {
            if cell_voltage == Self::CELL_VOLTAGE_ZERO {
                return Some(BmsFault::CellVoltageOpenWire(idx));
            }

            if cell_voltage > Self::CELL_VOLTAGE_SHORT_TO_SUPPLY {
                return Some(BmsFault::CellVoltageShortToSupply(idx));
            }

            if cell_voltage > Self::CELL_VOLTAGE_ZERO
                && cell_voltage < Self::CELL_VOLTAGE_SHORT_TO_GND_MAX
            {
                return Some(BmsFault::CellVoltageShortToGnd(idx));
            }

            if cell_voltage < Self::CELL_VOLTAGE_MIN_SAFE
                || cell_voltage > Self::CELL_VOLTAGE_MAX_SAFE
            {
                return Some(BmsFault::CellVoltageOutOfRange(idx));
            }
        }

        for (idx, &temp_c) in self.temperatures_c.iter().enumerate() {
            if temp_c == Self::TEMP_SENSOR_DISCONNECTED {
                return Some(BmsFault::TempSensorOpenWire(idx));
            }

            if temp_c > Self::TEMP_SHORT_TO_SUPPLY {
                return Some(BmsFault::TempSensorShortToSupply(idx));
            }

            if temp_c < Self::TEMP_SHORT_TO_GND_MIN {
                return Some(BmsFault::TempSensorShortToGnd(idx));
            }

            if temp_c < Self::TEMP_MIN_SAFE || temp_c > Self::TEMP_MAX_SAFE {
                return Some(BmsFault::TempOutOfRange(idx));
            }
        }

        None
    }
}
