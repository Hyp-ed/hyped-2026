use core::{cell::RefCell, num::TryFromIntError};

use defmt_rtt as _;
use embassy_stm32::{
    bind_interrupts, peripherals,
    sdmmc::{self, DataBlock, Sdmmc},
    time::{mhz, Hertz},
    Config,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embedded_sdmmc::{BlockCount, Mode, TimeSource, VolumeIdx, VolumeManager};
use panic_probe as _;

use embedded_sdmmc::blockdevice::{BlockDevice, BlockIdx};

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

const BLOCK_SIZE: u32 = 512;
pub static LOG_CHANNEL: Channel<ThreadModeRawMutex, &'static str, 4> = Channel::new();

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

pub struct HypedSdmmcTimeSource;

impl TimeSource for HypedSdmmcTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        // TODO: Use RTC
        embedded_sdmmc::Timestamp {
            year_since_1970: 53, // 2023
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[derive(Debug, Format)]
pub enum HypedSdmmcError {
    Sdmmc(sdmmc::Error),
    DriveSizeTooLarge,
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
}

impl BlockDevice for HypedSdmmc {
    type Error = HypedSdmmcError;

    // TODO: fork the embassy-sdmmc crate to add async read/write support
    // fn read(
    //     &self,
    //     blocks: &mut [embedded_sdmmc::Block],
    //     start_block_idx: BlockIdx,
    // ) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send {
    // }

    async fn read(
        &self,
        blocks: &mut [embedded_sdmmc::Block],
        start_block_idx: BlockIdx,
    ) -> Result<(), Self::Error> {
        for (i, block) in blocks.iter_mut().enumerate() {
            let block_id = start_block_idx.0 + i as u32;
            let mut data_block = DataBlock(block.contents);

            self.sdmmc
                .borrow_mut()
                .read_block(block_id, &mut data_block)
                .await?;
        }
        Ok(())
    }

    async fn write(
        &self,
        blocks: &[embedded_sdmmc::Block],
        start_block_idx: BlockIdx,
    ) -> Result<(), Self::Error> {
        for (i, block) in blocks.iter().enumerate() {
            let block_id = start_block_idx.0 + i as u32;
            let mut data_block = DataBlock(block.contents);

            self.sdmmc
                .borrow_mut()
                .write_block(block_id, &mut data_block)
                .await?;
        }

        Ok(())
    }

    async fn num_blocks(&self) -> Result<embedded_sdmmc::BlockCount, Self::Error> {
        let count = u32::try_from(self.sdmmc.borrow().card()?.size())? / BLOCK_SIZE;

        Ok(BlockCount(count))
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
    let hyped_sdmmc = HypedSdmmc::new().await;
    let volume_manager = VolumeManager::new(hyped_sdmmc, HypedSdmmcTimeSource);
    let volume0 = match volume_manager.open_volume(VolumeIdx(0)) {
        Ok(res) => res,
        Err(e) => {
            // TODO: sort out logging
            // defmt::error!("Sdmmc: Failed to initialize the volume: {:#?}", e);
            return;
        }
    };

    let root_dir = match volume0.open_root_dir() {
        Ok(res) => res,
        Err(e) => {
            // defmt::error!("Sdmmc: Failed to open root dir: {:#?}", e);
            return;
        }
    };

    let logging_file = match root_dir.open_file_in_dir("log.txt", Mode::ReadWriteCreateOrAppend) {
        Ok(res) => res,
        Err(e) => {
            // defmt::error!("Sdmmc: Failed to open the logs: {:#?}", e);
            return;
        }
    };

    loop {
        let message = LOG_CHANNEL.receiver().receive().await;

        logging_file.write(message.as_bytes()).unwrap_or_else(|e| {
            defmt::error!("Sdmmc: Failed to log the message: {}, ignoring...", message);
        });
    }
}
