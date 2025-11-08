use core::{cell::RefCell, num::TryFromIntError};

use defmt_rtt as _;
use dvida_serialize::{DvDeserialize, DvSerialize};
use embassy_stm32::{
    bind_interrupts, peripherals,
    sdmmc::{self, DataBlock, Sdmmc},
    time::{mhz, Hertz},
    Config,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Instant;
use panic_probe as _;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use crate::sdmmc::logging::{
    DescriptorBlock, LogEntryDescriptor, Message, LOG_ENTRY_DESCRIPTOR_SIZE, MAGIC, MESSAGE_SIZE,
};

pub mod logging;

const BLOCK_SIZE: usize = 512;
pub static LOG_CHANNEL: Channel<ThreadModeRawMutex, [u8; 40], 4> = Channel::new();

/*
use embedded_sdmmc::VolumeManager;
fn init_volume() {
    let hyped_sdmmc = embassy_futures::block_on(HypedSdmmc::new());
    let mut volume_manager = VolumeManager::new(hyped_sdmmc, HypedSdmmcTimeSource);
}
*/

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
});

#[derive(Debug, Format)]
pub enum HypedSdmmcError {
    Sdmmc(sdmmc::Error),
    DriveSizeTooLarge,
    SerializationError,
    // Other error variants can be added here
}

impl From<sdmmc::Error> for HypedSdmmcError {
    fn from(err: sdmmc::Error) -> Self {
        HypedSdmmcError::Sdmmc(err)
    }
}

impl From<TryFromIntError> for HypedSdmmcError {
    fn from(_err: TryFromIntError) -> Self {
        // Map the integer conversion error to a suitable HypedSdmmcError variant
        // Here we just use Sdmmc variant for simplicity
        HypedSdmmcError::DriveSizeTooLarge
    }
}

pub struct HypedSdmmc {
    sdmmc: RefCell<Sdmmc<'static, peripherals::SDMMC1, peripherals::DMA2_CH3>>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BlockIdx(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BlockCount(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Block(pub [u8; BLOCK_SIZE]);

impl HypedSdmmc {
    pub async fn new_with_peripherals(p: embassy_stm32::Peripherals) -> Self {
        let sdmmc = initialize_sdmmc(p).await;

        HypedSdmmc {
            sdmmc: RefCell::new(sdmmc),
        }
    }

    pub async fn new() -> Self {
        let config = create_config();
        let p = embassy_stm32::init(config);

        HypedSdmmc::new_with_peripherals(p).await
    }

    pub async fn read(
        &self,
        blocks: &mut [DataBlock],
        start_block_idx: BlockIdx,
    ) -> Result<(), HypedSdmmcError> {
        for (i, block) in blocks.iter_mut().enumerate() {
            let block_id = start_block_idx.0 + i as u32;

            self.sdmmc.borrow_mut().read_block(block_id, block).await?;
        }
        Ok(())
    }

    pub async fn write(
        &self,
        blocks: &[DataBlock],
        start_block_idx: BlockIdx,
    ) -> Result<(), HypedSdmmcError> {
        for (i, block) in blocks.iter().enumerate() {
            let block_id = start_block_idx.0 + i as u32;

            self.sdmmc.borrow_mut().write_block(block_id, block).await?;
        }

        Ok(())
    }

    pub async fn num_blocks(&self) -> Result<BlockCount, HypedSdmmcError> {
        let count = u32::try_from(self.sdmmc.borrow().card()?.size())? / BLOCK_SIZE as u32;

        Ok(BlockCount(count))
    }

    pub async fn init(&self) -> Result<DescriptorBlock, HypedSdmmcError> {
        let mut buffer = [DataBlock([0 as u8; BLOCK_SIZE]); 1];
        self.read(&mut buffer, BlockIdx(0)).await?;

        let mut descriptor_block =
            DescriptorBlock::deserialize(dvida_serialize::Endianness::Little, &buffer[0].0)
                .map_err(|_| HypedSdmmcError::SerializationError)?
                .0;

        // zero the buffer
        buffer[0].0.fill(0);

        // initialize the thing if empty, else read it
        if descriptor_block.magic != MAGIC {
            descriptor_block = DescriptorBlock::default();
            descriptor_block
                .serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
                .map_err(|_| HypedSdmmcError::SerializationError)?;

            self.write(&mut buffer, BlockIdx(0)).await?;
        }

        buffer[0].0.fill(0);

        // update the previous log entry
        self.read(&mut buffer, BlockIdx(descriptor_block.last_log_start))
            .await?;

        let mut prev_log_entry =
            LogEntryDescriptor::deserialize(dvida_serialize::Endianness::Little, &buffer[0].0)
                .map_err(|_| HypedSdmmcError::SerializationError)?
                .0;

        prev_log_entry.next_log_start = descriptor_block.first_free_block;

        buffer[0].0.fill(0);

        prev_log_entry
            .serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
            .map_err(|_| HypedSdmmcError::SerializationError)?;

        self.write(&mut buffer, BlockIdx(descriptor_block.last_log_start))
            .await?;

        // create a log entry
        buffer[0].0.fill(0);

        let log_entry = LogEntryDescriptor {
            last_log_start: descriptor_block.last_log_start,
            next_log_start: 0,
        };

        log_entry
            .serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
            .map_err(|_| HypedSdmmcError::SerializationError)?;

        self.write(&mut buffer, BlockIdx(descriptor_block.first_free_block))
            .await?;

        // update descriptor block
        buffer[0].0.fill(0);

        let descriptor_block_clone = descriptor_block.clone();
        descriptor_block.last_log_start = descriptor_block.first_free_block;
        descriptor_block.first_free_block += 1;
        descriptor_block.current_trial_id += 1;

        descriptor_block
            .serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
            .map_err(|_| HypedSdmmcError::SerializationError)?;

        self.write(&mut buffer, BlockIdx(0)).await?;

        Ok(descriptor_block_clone)
    }
}

// Adapted from https://github.com/embassy-rs/embassy/blob/bcebe4c4d5b597da0b8741916e450c46e6fef06e/examples/stm32f7/src/bin/sdmmc.rs
fn create_config() -> Config {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL216,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 216 / 2 = 216Mhz
            divq: Some(PllQDiv::DIV9), // 8mhz / 4 * 216 / 9 = 48Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    config
}

async fn initialize_sdmmc(
    p: embassy_stm32::Peripherals,
) -> Sdmmc<'static, peripherals::SDMMC1, peripherals::DMA2_CH3> {
    let mut sdmmc = Sdmmc::new_4bit(
        p.SDMMC1,
        Irqs,
        p.DMA2_CH3,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    unwrap!(sdmmc.init_card(mhz(25)).await);
    sdmmc
}

#[embassy_executor::task]
pub async fn sdmmc_task() {
    let sdmmc = HypedSdmmc::new().await;
    let descriptor_block = match sdmmc.init().await {
        Ok(b) => b,
        Err(e) => {
            defmt::error!("Cannot Initialize sdmmc: {}", e);
            return;
        }
    };

    let start_time = Instant::now();
    let mut buffer = [DataBlock([0 as u8; BLOCK_SIZE]); 1];

    let mut blocks_written_count: u32 = 0;
    let mut bytes_written_count: u32 = LOG_ENTRY_DESCRIPTOR_SIZE;

    loop {
        let msg = LOG_CHANNEL.receiver().receive().await;
        let time = Instant::now() - start_time;
        let time_sec = time.as_secs();
        let time_milli = time.as_millis() % 1000;
        let time_micro = time.as_micros() % 1000;

        // write the message
        buffer[0].fill(0);
        if let Err(_) = sdmmc
            .read(
                &mut buffer,
                BlockIdx(descriptor_block.first_free_block + blocks_written_count),
            )
            .await
        {
            continue;
        }

        let message = Message {
            time_sec,
            time_milli,
            time_micro,
            message: msg,
        };

        if let Err(_) = message.serialize(
            dvida_serialize::Endianness::Little,
            &mut buffer[0].0[bytes_written_count as usize..],
        ) {
            continue;
        }

        if let Err(_) = sdmmc
            .write(
                &mut buffer,
                BlockIdx(descriptor_block.first_free_block + blocks_written_count),
            )
            .await
        {
            continue;
        }

        bytes_written_count += MESSAGE_SIZE;
        if bytes_written_count >= 512 {
            // update the descriptor
            bytes_written_count %= 512;
            blocks_written_count += 1;

            buffer[0].fill(0);

            if let Err(_) = sdmmc.read(&mut buffer, BlockIdx(0)).await {
                continue;
            }

            let mut descriptor = match DescriptorBlock::deserialize(
                dvida_serialize::Endianness::Little,
                &mut buffer[0].0,
            ) {
                Ok(d) => d.0,
                Err(_) => continue,
            };

            descriptor.first_free_block += 1;

            buffer[0].fill(0);

            if let Err(_) =
                descriptor.serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
            {
                continue;
            }

            if let Err(_) = sdmmc.write(&mut buffer, BlockIdx(0)).await {
                continue;
            }
        }
    }
}
