/// Driver for the TinyBMS s516 30A Battery Management System using UART.
/// Used to monitor battery status and health.
/// UART configuration: 115200 baud, 8 data bits, 1 stop bit, no parity, no flow control.
use defmt::Format;
use hyped_uart::{HypedUart, UartErr};

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
    const START_BYTE: u8 = 0xAA;

    const RESET_OR_CLEAN_EVENT_OR_STATUS: u8 = 0x02;
    const RESET_BMS: u8 = 0x05;

    const READ_REGISTERS: u8 = 0x03;

    const READ_VOLTAGE: u8 = 0x14;
    const READ_CURRENT: u8 = 0x15;
    const READ_MAX_CELL_VOLTAGE: u8 = 0x16;
    const READ_MIN_CELL_VOLTAGE: u8 = 0x17;
    const READ_TEMPERATURES: u8 = 0x1B;
    const READ_CELL_VOLTAGES: u8 = 0x1C;

    const ACK: u8 = 0x01;
    const NACK: u8 = 0x00;
}

/// CRC16 lookup table (MODBUS polynomial 0x8005).
const CRC_TABLE: [u16; 256] = [
    0x0000, 0xC0C1, 0xC181, 0x0140, 0xC301, 0x03C0, 0x0280, 0xC241, 0xC601, 0x06C0, 0x0780, 0xC741,
    0x0500, 0xC5C1, 0xC481, 0x0440, 0xCC01, 0x0CC0, 0x0D80, 0xCD41, 0x0F00, 0xCFC1, 0xCE81, 0x0E40,
    0x0A00, 0xCAC1, 0xCB81, 0x0B40, 0xC901, 0x09C0, 0x0880, 0xC841, 0xD801, 0x18C0, 0x1980, 0xD941,
    0x1B00, 0xDBC1, 0xDA81, 0x1A40, 0x1E00, 0xDEC1, 0xDF81, 0x1F40, 0xDD01, 0x1DC0, 0x1C80, 0xDC41,
    0x1400, 0xD4C1, 0xD581, 0x1540, 0xD701, 0x17C0, 0x1680, 0xD641, 0xD201, 0x12C0, 0x1380, 0xD341,
    0x1100, 0xD1C1, 0xD081, 0x1040, 0xF001, 0x30C0, 0x3180, 0xF141, 0x3300, 0xF3C1, 0xF281, 0x3240,
    0x3600, 0xF6C1, 0xF781, 0x3740, 0xF501, 0x35C0, 0x3480, 0xF441, 0x3C00, 0xFCC1, 0xFD81, 0x3D40,
    0xFF01, 0x3FC0, 0x3E80, 0xFE41, 0xFA01, 0x3AC0, 0x3B80, 0xFB41, 0x3900, 0xF9C1, 0xF881, 0x3840,
    0x2800, 0xE8C1, 0xE981, 0x2940, 0xEB01, 0x2BC0, 0x2A80, 0xEA41, 0xEE01, 0x2EC0, 0x2F80, 0xEF41,
    0x2D00, 0xEDC1, 0xEC81, 0x2C40, 0xE401, 0x24C0, 0x2580, 0xE541, 0x2700, 0xE7C1, 0xE681, 0x2640,
    0x2200, 0xE2C1, 0xE381, 0x2340, 0xE101, 0x21C0, 0x2080, 0xE041, 0xA001, 0x60C0, 0x6180, 0xA141,
    0x6300, 0xA3C1, 0xA281, 0x6240, 0x6600, 0xA6C1, 0xA781, 0x6740, 0xA501, 0x65C0, 0x6480, 0xA441,
    0x6C00, 0xACC1, 0xAD81, 0x6D40, 0xAF01, 0x6FC0, 0x6E80, 0xAE41, 0xAA01, 0x6AC0, 0x6B80, 0xAB41,
    0x6900, 0xA9C1, 0xA881, 0x6840, 0x7800, 0xB8C1, 0xB981, 0x7940, 0xBB01, 0x7BC0, 0x7A80, 0xBA41,
    0xBE01, 0x7EC0, 0x7F80, 0xBF41, 0x7D00, 0xBDC1, 0xBC81, 0x7C40, 0xB401, 0x74C0, 0x7580, 0xB541,
    0x7700, 0xB7C1, 0xB681, 0x7640, 0x7200, 0xB2C1, 0xB381, 0x7340, 0xB101, 0x71C0, 0x7080, 0xB041,
    0x5000, 0x90C1, 0x9181, 0x5140, 0x9301, 0x53C0, 0x5280, 0x9241, 0x9601, 0x56C0, 0x5780, 0x9741,
    0x5500, 0x95C1, 0x9481, 0x5440, 0x9C01, 0x5CC0, 0x5D80, 0x9D41, 0x5F00, 0x9FC1, 0x9E81, 0x5E40,
    0x5A00, 0x9AC1, 0x9B81, 0x5B40, 0x9901, 0x59C0, 0x5880, 0x9841, 0x8801, 0x48C0, 0x4980, 0x8941,
    0x4B00, 0x8BC1, 0x8A81, 0x4A40, 0x4E00, 0x8EC1, 0x8F81, 0x4F40, 0x8D01, 0x4DC0, 0x4C80, 0x8C41,
    0x4400, 0x84C1, 0x8581, 0x4540, 0x8701, 0x47C0, 0x4680, 0x8641, 0x8201, 0x42C0, 0x4380, 0x8341,
    0x4100, 0x81C1, 0x8081, 0x4040,
];

fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        let tmp = (byte ^ crc as u8) as usize;
        crc = (crc >> 8) ^ CRC_TABLE[tmp];
    }
    crc
}

pub struct BmsUart<U: HypedUart> {
    uart: U,
    cells_count: u16,
}

const BMS_TEMPERATURE_COUNT: usize = 3;
const CELL_COUNT_REGISTER: u8 = 53;

impl<U: HypedUart> BmsUart<U> {
    pub fn new(uart: U) -> Self {
        defmt::debug!("BMS: creating new UART instance");
        Self {
            uart,
            cells_count: 0,
        }
    }

    pub async fn init(&mut self) -> Result<(), UartErr> {
        defmt::info!("BMS: initializing");
        self.reset().await?;
        self.cells_count = self.read_cell_count().await?;
        defmt::info!("BMS: initialized with {} cells", self.cells_count);
        Ok(())
    }

    /// Writes a request frame: [0xAA, cmd, ...payload..., CRC_LSB, CRC_MSB].
    async fn send_request(&mut self, payload: &[u8]) -> Result<(), UartErr> {
        // Build the frame without CRC first, then compute and append.
        // Maximum frame we ever send is small, so a 16-byte stack buffer is enough.
        let mut frame: heapless::Vec<u8, 16> = heapless::Vec::new();
        frame
            .extend_from_slice(payload)
            .map_err(|_| UartErr::BufferOverflow)?;

        let crc = crc16(&frame);
        frame
            .push((crc & 0xFF) as u8)
            .map_err(|_| UartErr::BufferOverflow)?;
        frame
            .push((crc >> 8) as u8)
            .map_err(|_| UartErr::BufferOverflow)?;

        defmt::trace!("BMS UART TX: {} bytes", frame.len());
        self.uart.write(&frame).await
    }

    /// Reads exactly `n` bytes into `buf`.
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), UartErr> {
        self.uart.read(buf).await
    }

    /// Reads and validates a fixed-length response.
    ///
    /// Every response begins with [0xAA, status, cmd, ...].
    /// Returns the full response buffer (without the trailing CRC bytes) on success.
    ///
    /// `expected_cmd` – the command byte we expect back in position [2].
    /// `total_len`    – total number of bytes in the response (including 0xAA, status, cmd and CRC).
    async fn read_response<const N: usize>(
        &mut self,
        expected_cmd: u8,
    ) -> Result<[u8; N], UartErr> {
        defmt::trace!(
            "BMS UART: waiting for response to cmd=0x{:02X}",
            expected_cmd
        );

        let mut buf = [0u8; N];
        self.read_exact(&mut buf).await?;

        // Validate start byte.
        if buf[0] != BmsCmd::START_BYTE {
            defmt::error!("BMS UART: bad start byte 0x{:02X} (expected 0xAA)", buf[0]);
            return Err(UartErr::Unknown);
        }

        // Validate CRC over everything except the last two bytes.
        let computed = crc16(&buf[..N - 2]);
        let received = (buf[N - 1] as u16) << 8 | buf[N - 2] as u16;
        if computed != received {
            defmt::error!(
                "BMS UART: CRC mismatch (computed=0x{:04X}, received=0x{:04X})",
                computed,
                received
            );
            return Err(UartErr::CrcError);
        }

        // Check for NACK [0xAA, 0x00, cmd, ERROR, CRC:LSB, CRC:MSB].
        if buf[1] == BmsCmd::NACK {
            defmt::error!(
                "BMS UART: NACK for cmd=0x{:02X}, error=0x{:02X}",
                expected_cmd,
                buf[3]
            );
            return Err(UartErr::Unknown);
        }

        // Validate ACK byte and command echo.
        if buf[1] != BmsCmd::ACK || buf[2] != expected_cmd {
            defmt::error!(
                "BMS UART: unexpected response status=0x{:02X} cmd=0x{:02X}",
                buf[1],
                buf[2]
            );
            return Err(UartErr::Unknown);
        }

        defmt::trace!("BMS UART: valid response for cmd=0x{:02X}", expected_cmd);
        Ok(buf)
    }

    pub async fn read_voltage(&mut self) -> Result<f32, UartErr> {
        // Request: [0xAA, 0x14, CRC:LSB, CRC:MSB]
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_VOLTAGE])
            .await?;

        // Response: [0xAA, 0x01, 0x14, DATA(4 bytes FLOAT), CRC:LSB, CRC:MSB] = 8 bytes
        let buf: [u8; 8] = self.read_response(BmsCmd::READ_VOLTAGE).await?;
        let voltage = f32::from_le_bytes([buf[3], buf[4], buf[5], buf[6]]);
        defmt::debug!("BMS: voltage = {}V", voltage);
        Ok(voltage)
    }

    pub async fn read_current(&mut self) -> Result<f32, UartErr> {
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_CURRENT])
            .await?;

        // Response: [0xAA, 0x01, 0x15, DATA(4 bytes FLOAT), CRC:LSB, CRC:MSB] = 8 bytes
        let buf: [u8; 8] = self.read_response(BmsCmd::READ_CURRENT).await?;
        let current = f32::from_le_bytes([buf[3], buf[4], buf[5], buf[6]]);
        defmt::debug!("BMS: current = {}A", current);
        Ok(current)
    }

    pub async fn read_max_cell_voltage(&mut self) -> Result<u16, UartErr> {
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_MAX_CELL_VOLTAGE])
            .await?;

        // Response: [0xAA, 0x01, 0x16, DATA:LSB, DATA:MSB, CRC:LSB, CRC:MSB] = 7 bytes
        let buf: [u8; 7] = self.read_response(BmsCmd::READ_MAX_CELL_VOLTAGE).await?;
        let max_mv = u16::from_le_bytes([buf[3], buf[4]]);
        defmt::debug!("BMS: max cell voltage = {}mV", max_mv);
        Ok(max_mv)
    }

    pub async fn read_min_cell_voltage(&mut self) -> Result<u16, UartErr> {
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_MIN_CELL_VOLTAGE])
            .await?;

        // Response: [0xAA, 0x01, 0x17, DATA:LSB, DATA:MSB, CRC:LSB, CRC:MSB] = 7 bytes
        let buf: [u8; 7] = self.read_response(BmsCmd::READ_MIN_CELL_VOLTAGE).await?;
        let min_mv = u16::from_le_bytes([buf[3], buf[4]]);
        defmt::debug!("BMS: min cell voltage = {}mV", min_mv);
        Ok(min_mv)
    }

    pub async fn read_temperatures(&mut self) -> Result<[i16; BMS_TEMPERATURE_COUNT], UartErr> {
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_TEMPERATURES])
            .await?;

        // Response: [0xAA, 0x1B, PL, D1:LSB, D1:MSB, D2:LSB, D2:MSB, D3:LSB, D3:MSB, CRC:LSB, CRC:MSB]
        // PL = 6 (3 x INT_16), total = 11 bytes.
        let buf: [u8; 11] = self.read_response(BmsCmd::READ_TEMPERATURES).await?;

        let temps = [
            i16::from_le_bytes([buf[3], buf[4]]),
            i16::from_le_bytes([buf[5], buf[6]]),
            i16::from_le_bytes([buf[7], buf[8]]),
        ];
        defmt::debug!(
            "BMS: temperatures (internal, ext1, ext2) = {}°C, {}°C, {}°C",
            temps[0],
            temps[1],
            temps[2]
        );
        Ok(temps)
    }

    pub async fn read_cell_voltages(&mut self) -> Result<heapless::Vec<u16, 32>, UartErr> {
        self.send_request(&[BmsCmd::START_BYTE, BmsCmd::READ_CELL_VOLTAGES])
            .await?;

        // The BMS sends a variable-length response:
        // [0xAA, 0x1C, PL, DATA1:LSB, DATA1:MSB, ..., DATAn:LSB, DATAn:MSB, CRC:LSB, CRC:MSB]
        // We first read the 3-byte header to get PL, then read the rest.

        let mut header = [0u8; 3];
        self.read_exact(&mut header).await?;

        if header[0] != BmsCmd::START_BYTE {
            defmt::error!("BMS UART: bad start byte in cell voltages response");
            return Err(UartErr::Unknown);
        }
        if header[1] != BmsCmd::READ_CELL_VOLTAGES {
            defmt::error!("BMS UART: unexpected cmd in cell voltages response");
            return Err(UartErr::Unknown);
        }

        let payload_len = header[2] as usize;
        let cell_count = payload_len / 2;

        // Read payload + 2 CRC bytes.
        let tail_len = payload_len + 2;
        let mut tail: heapless::Vec<u8, 68> = heapless::Vec::new(); // 32 cells * 2 + 4 overhead
        tail.resize_default(tail_len)
            .map_err(|_| UartErr::BufferOverflow)?;
        self.read_exact(&mut tail).await?;

        // Validate CRC over [header || payload] (everything except last 2 bytes of tail).
        let mut crc_data: heapless::Vec<u8, 72> = heapless::Vec::new();
        crc_data
            .extend_from_slice(&header)
            .map_err(|_| UartErr::BufferOverflow)?;
        crc_data
            .extend_from_slice(&tail[..payload_len])
            .map_err(|_| UartErr::BufferOverflow)?;

        let computed = crc16(&crc_data);
        let received = (tail[tail_len - 1] as u16) << 8 | tail[tail_len - 2] as u16;
        if computed != received {
            defmt::error!("BMS UART: CRC mismatch in cell voltages response");
            return Err(UartErr::CrcError);
        }

        let mut voltages = heapless::Vec::<u16, 32>::new();
        for i in 0..cell_count.min(self.cells_count as usize) {
            let lsb = tail[i * 2];
            let msb = tail[i * 2 + 1];
            let val = u16::from_le_bytes([lsb, msb]);
            defmt::trace!("BMS: cell[{}] voltage = {}mV", i, val);
            voltages.push(val).map_err(|_| {
                defmt::error!("BMS: cell voltage buffer overflow at cell {}", i);
                UartErr::BufferOverflow
            })?;
        }

        defmt::debug!("BMS: read {} cell voltages", voltages.len());
        Ok(voltages)
    }

    pub async fn reset(&mut self) -> Result<(), UartErr> {
        defmt::info!("BMS: sending reset");
        // Request: [0xAA, 0x02, OPTION=0x05, CRC:LSB, CRC:MSB]
        self.send_request(&[
            BmsCmd::START_BYTE,
            BmsCmd::RESET_OR_CLEAN_EVENT_OR_STATUS,
            BmsCmd::RESET_BMS,
        ])
        .await?;

        // ACK response: [0xAA, 0x01, 0x02, CRC:LSB, CRC:MSB] = 5 bytes
        self.read_response::<5>(BmsCmd::RESET_OR_CLEAN_EVENT_OR_STATUS)
            .await
            .map(|_| {
                defmt::info!("BMS: reset acknowledged");
            })
    }

    pub async fn read_cell_count(&mut self) -> Result<u16, UartErr> {
        defmt::debug!("BMS: reading cell count");
        // MODBUS-compatible read: [0xAA, 0x03, ADDR:MSB, ADDR:LSB, 0x00, RL, CRC:LSB, CRC:MSB]
        // Register 53 (0x0035) = Number Of Detected Cells, read 1 register.
        self.send_request(&[
            BmsCmd::START_BYTE,
            BmsCmd::READ_REGISTERS,
            0x00,
            CELL_COUNT_REGISTER,
            0x00,
            0x01,
        ])
        .await?;

        // Response: [0xAA, 0x03, PL=2, DATA:MSB, DATA:LSB, CRC:LSB, CRC:MSB] = 7 bytes
        // Note: MODBUS response is big-endian per spec section 1.1.6.
        let buf: [u8; 7] = self.read_response(BmsCmd::READ_REGISTERS).await?;
        let count = u16::from_be_bytes([buf[3], buf[4]]);
        defmt::info!("BMS: cell count = {}", count);
        Ok(count)
    }

    pub async fn read_battery_data(&mut self) -> Result<BatteryData, UartErr> {
        defmt::debug!("BMS: reading full battery data");
        let voltage = self.read_voltage().await?;
        let current = self.read_current().await?;
        let max_cell_mv = self.read_max_cell_voltage().await?;
        let min_cell_mv = self.read_min_cell_voltage().await?;
        let temperatures_c = self.read_temperatures().await?;
        let cell_voltages_mv = self.read_cell_voltages().await?;

        let data = BatteryData {
            voltage,
            current,
            max_cell_mv,
            min_cell_mv,
            temperatures_c,
            cell_voltages_mv,
        };
        defmt::info!(
            "BMS: battery data read complete — {}V, {}A, max_cell={}mV, min_cell={}mV",
            data.voltage,
            data.current,
            data.max_cell_mv,
            data.min_cell_mv
        );
        Ok(data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum BmsFault {
    CellVoltageOpenWire(usize),
    CellVoltageShortToSupply(usize),
    CellVoltageShortToGnd(usize),
    CellVoltageOutOfRange(usize),

    TempSensorOpenWire(usize),
    TempSensorShortToSupply(usize),
    TempSensorShortToGnd(usize),
    TempOutOfRange(usize),
}

impl BatteryData {
    const CELL_VOLTAGE_ZERO: u16 = 0;
    const CELL_VOLTAGE_SHORT_TO_GND_MAX: u16 = 100;
    const CELL_VOLTAGE_MIN_SAFE: u16 = 2000;
    const CELL_VOLTAGE_MAX_SAFE: u16 = 4500;
    const CELL_VOLTAGE_SHORT_TO_SUPPLY: u16 = 5000;

    const TEMP_SENSOR_DISCONNECTED: i16 = -32768;
    const TEMP_MIN_SAFE: i16 = -40;
    const TEMP_SHORT_TO_GND_MIN: i16 = -50;
    const TEMP_MAX_SAFE: i16 = 90;
    const TEMP_SHORT_TO_SUPPLY: i16 = 150;

    pub fn check_faults(&self) -> Option<BmsFault> {
        for (idx, &cell_voltage) in self.cell_voltages_mv.iter().enumerate() {
            if cell_voltage == Self::CELL_VOLTAGE_ZERO {
                defmt::warn!("BMS fault: cell[{}] open wire (voltage = 0mV)", idx);
                return Some(BmsFault::CellVoltageOpenWire(idx));
            }
            if cell_voltage > Self::CELL_VOLTAGE_SHORT_TO_SUPPLY {
                defmt::warn!(
                    "BMS fault: cell[{}] short to supply ({}mV > {}mV)",
                    idx,
                    cell_voltage,
                    Self::CELL_VOLTAGE_SHORT_TO_SUPPLY
                );
                return Some(BmsFault::CellVoltageShortToSupply(idx));
            }
            if cell_voltage > Self::CELL_VOLTAGE_ZERO
                && cell_voltage < Self::CELL_VOLTAGE_SHORT_TO_GND_MAX
            {
                defmt::warn!(
                    "BMS fault: cell[{}] short to GND ({}mV < {}mV)",
                    idx,
                    cell_voltage,
                    Self::CELL_VOLTAGE_SHORT_TO_GND_MAX
                );
                return Some(BmsFault::CellVoltageShortToGnd(idx));
            }
            if !(Self::CELL_VOLTAGE_MIN_SAFE..=Self::CELL_VOLTAGE_MAX_SAFE).contains(&cell_voltage)
            {
                defmt::warn!(
                    "BMS fault: cell[{}] voltage out of safe range ({}mV, safe={}..{}mV)",
                    idx,
                    cell_voltage,
                    Self::CELL_VOLTAGE_MIN_SAFE,
                    Self::CELL_VOLTAGE_MAX_SAFE
                );
                return Some(BmsFault::CellVoltageOutOfRange(idx));
            }
        }

        for (idx, &temp_c) in self.temperatures_c.iter().enumerate() {
            if temp_c == Self::TEMP_SENSOR_DISCONNECTED {
                defmt::warn!("BMS fault: temp sensor[{}] disconnected (open wire)", idx);
                return Some(BmsFault::TempSensorOpenWire(idx));
            }
            if temp_c > Self::TEMP_SHORT_TO_SUPPLY {
                defmt::warn!(
                    "BMS fault: temp sensor[{}] short to supply ({}°C > {}°C)",
                    idx,
                    temp_c,
                    Self::TEMP_SHORT_TO_SUPPLY
                );
                return Some(BmsFault::TempSensorShortToSupply(idx));
            }
            if temp_c < Self::TEMP_SHORT_TO_GND_MIN {
                defmt::warn!(
                    "BMS fault: temp sensor[{}] short to GND ({}°C < {}°C)",
                    idx,
                    temp_c,
                    Self::TEMP_SHORT_TO_GND_MIN
                );
                return Some(BmsFault::TempSensorShortToGnd(idx));
            }
            if !(Self::TEMP_MIN_SAFE..=Self::TEMP_MAX_SAFE).contains(&temp_c) {
                defmt::warn!(
                    "BMS fault: temp sensor[{}] out of safe range ({}°C, safe={}..{}°C)",
                    idx,
                    temp_c,
                    Self::TEMP_MIN_SAFE,
                    Self::TEMP_MAX_SAFE
                );
                return Some(BmsFault::TempOutOfRange(idx));
            }
        }

        defmt::trace!("BMS: no faults detected");
        None
    }
}
