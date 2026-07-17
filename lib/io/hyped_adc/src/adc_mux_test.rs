#![no_std]
#![no_main]

use hyped_adc::HypedAdc;
use core::cell::RefCell;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{i2c::I2c, mode::Blocking, time::Hertz};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

use hyped_adc::adc_mux::{AdcChannelAddress, ADCMuxChannel, ADC_MUX_ADDRESS};
const ADC_ADDR: u8 = ADC_MUX_ADDRESS;

type I2c1Bus = Mutex<NoopRawMutex, RefCell<I2c<'static, Blocking>>>;

// driver write 0x1D
const ADC_ADDR: u8 = 0x1D;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz(200_000), Default::default());

    static I2C_BUS: StaticCell<I2c1Bus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(RefCell::new(i2c)));

    info!("ADC128D818 test starting. I2C addr=0x{:02x}", ADC_ADDR);

    // Safe
    Timer::after_millis(35).await;

    let channels: [AdcChannelAddress; 8] = [
        AdcChannelAddress::AdcChannel0,
        AdcChannelAddress::AdcChannel1,
        AdcChannelAddress::AdcChannel2,
        AdcChannelAddress::AdcChannel3,
        AdcChannelAddress::AdcChannel4,
        AdcChannelAddress::AdcChannel5,
        AdcChannelAddress::AdcChannel6,
        AdcChannelAddress::AdcChannel7,
    ];

    loop {
    info!("polling...");

    i2c_bus.lock(|_cell| {
        // TODO: once ADCMuxChannel::new takes &mut I2C (borrow)
        for (i, _ch) in channels.iter().enumerate() {
            info!("TODO: read channel {}", i);
        }
    });

    Timer::after_millis(200).await;
}
}