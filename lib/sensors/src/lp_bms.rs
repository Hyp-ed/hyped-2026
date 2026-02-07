use core::task::Poll;

/// Driver for the TinyBMS s516 30A Battery Management System using CAN.
/// Used to monitor battery status and health.
/// For now it is assumed that the BMS communicates with another bus
use defmt::Format;
use embassy_stm32::can::CanRx;
use embassy_time::{Duration, Instant};
use hyped_can::{CanError, HypedCanFrame, HypedCanRx, HypedCanTx};

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
    can: CanRx<'static>,
}

pub struct ReadResponseFuture<'a, T: HypedCanTx + HypedCanRx + 'a> {
    can: &'a mut T,
    response_id: u32,
    expected_cmd: u8,
    start: Instant,
}

impl<'a, T: HypedCanTx + HypedCanRx> core::future::Future for ReadResponseFuture<'a, T> {
    type Output = Result<[u8; 8], CanError>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.get_mut();
        let envelope = this.can.read_frame()?;
        if envelope.frame.can_id == this.response_id && envelope.frame.data[1] == this.expected_cmd
        {
            Poll::Ready(Ok(envelope.frame.data))
        } else {
            let elapsed = Instant::now() - this.start;
            if elapsed > Duration::from_millis(1000) {
                return Poll::Ready(Err(CanError::Timeout));
            }
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub const BMS_NODE_ID: u8 = 0x01;
pub const BMS_REQUEST_ID: u32 = 0x400 | BMS_NODE_ID as u32;
pub const BMS_RESPONSE_ID: u32 = 0x500 | BMS_NODE_ID as u32;

impl<'a, T: HypedCanTx + HypedCanRx> Bms<'a, T> {
    const REQUEST_ID: u32 = 0x400 | BMS_NODE_ID as u32;
    const RESPONSE_ID: u32 = 0x500 | BMS_NODE_ID as u32;

    fn send_simple_request(&mut self, cmd: u8) -> Result<(), CanError> {
        let frame = HypedCanFrame::new(Self::REQUEST_ID, [cmd, 0, 0, 0, 0, 0, 0, 0]);
        self.can.write_frame(&frame)
    }

    async fn read_response(&mut self, expected_cmd: u8) -> Result<[u8; 8], CanError> {
        let future = ReadResponseFuture {
            can: self.can,
            response_id: Self::RESPONSE_ID,
            expected_cmd,
            start: Instant::now(),
        };
        future.await
    }

    pub async fn read_voltage(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(0x14)?;
        let data = self.read_response(0x14).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_current(&mut self) -> Result<f32, CanError> {
        self.send_simple_request(0x15)?;
        let data = self.read_response(0x15).await?;
        Ok(f32::from_le_bytes([data[2], data[3], data[4], data[5]]))
    }

    pub async fn read_max_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(0x16)?;
        let data = self.read_response(0x16).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_min_cell_voltage(&mut self) -> Result<u16, CanError> {
        self.send_simple_request(0x17)?;
        let data = self.read_response(0x17).await?;
        Ok(u16::from_le_bytes([data[2], data[3]]))
    }

    pub async fn read_temperatures(&mut self) -> Result<[i16; 3], CanError> {
        self.send_simple_request(0x1B)?;
        let mut temps = [0i16; 3];
        for temp in &mut temps {
            let data = self.read_response(0x1B).await?;
            *temp = i16::from_le_bytes([data[2], data[3]]);
        }
        Ok(temps)
    }

    pub fn read_cell_voltages(&mut self) -> Result<heapless::Vec<u16, 32>, CanError> {
        self.send_simple_request(0x1C)?;
        let mut voltages = heapless::Vec::<u16, 32>::new();

        let start = Instant::now();
        loop {
            if Instant::now() - start > Duration::from_millis(1000) {
                return Err(CanError::Full);
            }

            match self.can.read_frame() {
                Ok(envelope)
                    if envelope.frame.can_id == Self::RESPONSE_ID
                        && envelope.frame.data[1] == 0x1C =>
                {
                    let data = envelope.frame.data;
                    let val = u16::from_le_bytes([data[2], data[3]]);
                    voltages.push(val).map_err(|_| CanError::BufferOverflow)?;
                }
                Ok(_) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(voltages)
    }
}

impl<'a, T: HypedCanTx + HypedCanRx> Bms<'a, T> {
    pub fn new(can: &'a mut T) -> Self {
        Bms { can }
    }

    pub async fn read_battery_data(&mut self) -> Result<BatteryData, CanError> {
        let voltage = self.read_voltage().await?;
        let current = self.read_current().await?;
        let max_cell_mv = self.read_max_cell_voltage().await?;
        let min_cell_mv = self.read_min_cell_voltage().await?;
        let temperatures_c = self.read_temperatures().await?;
        let cell_voltages_mv = self.read_cell_voltages()?;

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

impl<'a, T: HypedCanTx + HypedCanRx> Bms<'a, T> {
    pub fn init(&mut self) {}
}
