//! LED Driver Module
//!
//! This module provides an interface to control the LP5036 LED driver IC via I2C.
//! The LP5036 is a 36-channel, constant-current LED driver with pulse width modulation (PWM) control.
//!
//! Link to Led Driver: https://www.ti.com/lit/ds/symlink/lp5036.pdf?HQS=dis-mous-null-mousermode-dsf-pf-null-wwe&ts=1698441544495&ref_url=https%253A%252F%252Fwww.mouser.co.uk%252F

#![cfg_attr(not(test), no_std)]

use defmt::Format;
use hyped_i2c::{HypedI2c, I2cError};

// ============================================================================
// I2C Device Addresses
// ============================================================================

/// LP5036 default I2C address
const LP5036_DEFAULT_ADDRESS: u8 = 0x30;

// ============================================================================
// Register Addresses
// ============================================================================

/// Device configuration register 0
const DEVICE_CONFIG_0: u8 = 0x00;
/// Device configuration register 1
const DEVICE_CONFIG_1: u8 = 0x01;
/// LED enable register 0
const LED_CONFIG_0: u8 = 0x02;
/// LED enable register 1
const LED_CONFIG_1: u8 = 0x03;
/// LED enable register 2
const LED_CONFIG_2: u8 = 0x04;
/// Bank A PWM register
const BANK_A_PWM: u8 = 0x05;
/// Bank B PWM register
const BANK_B_PWM: u8 = 0x06;
/// Bank C PWM register
const BANK_C_PWM: u8 = 0x07;
/// LED 0-11 brightness (0x08 - 0x13)
const LED_BRIGHTNESS_BASE: u8 = 0x08;
/// Bank A color register
const BANK_A_COLOR: u8 = 0x14;
/// Bank B color register
const BANK_B_COLOR: u8 = 0x15;
/// Bank C color register
const BANK_C_COLOR: u8 = 0x16;
/// Reset register
const RESET: u8 = 0x17;

// ============================================================================
// Configuration Constants
// ============================================================================

/// Device configuration register 0 settings
const DEVICE_CONFIG_0_SETTINGS: u8 = 0x40; // ChipEnable = 1
/// Reset value to trigger device reset
const RESET_VALUE: u8 = 0xFF;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Copy, Format)]
pub enum LedDriverError {
    I2cError(I2cError),
    InvalidChannel,
    InvalidBrightness,
}

// ============================================================================
// LED Channel Enumeration
// ============================================================================

/// LED channels available on the LP5036 (0-35)
#[derive(Debug, Clone, Copy, Format, PartialEq)]
pub enum LedChannel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
    Channel4,
    Channel5,
    Channel6,
    Channel7,
    Channel8,
    Channel9,
    Channel10,
    Channel11,
    Channel12,
    Channel13,
    Channel14,
    Channel15,
    Channel16,
    Channel17,
    Channel18,
    Channel19,
    Channel20,
    Channel21,
    Channel22,
    Channel23,
    Channel24,
    Channel25,
    Channel26,
    Channel27,
    Channel28,
    Channel29,
    Channel30,
    Channel31,
    Channel32,
    Channel33,
    Channel34,
    Channel35,
}

impl LedChannel {
    /// Get the channel index (0-35)
    pub fn index(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// Bank Enumeration
// ============================================================================

/// LED banks for group PWM control
#[derive(Debug, Clone, Copy, Format)]
pub enum LedBank {
    BankA,
    BankB,
    BankC,
}

// ============================================================================
// LED Driver Structure
// ============================================================================

/// LED Driver for the LP5036 LED controller
/// Controls up to 36 LED channels with individual brightness and bank-level PWM control
pub struct LedDriver<'a, T: HypedI2c> {
    i2c: &'a mut T,
    device_address: u8,
}

impl<'a, T: HypedI2c> LedDriver<'a, T> {
    /// Create a new LED driver instance and initialize the device
    pub fn new(i2c: &'a mut T) -> Result<Self, LedDriverError> {
        Self::new_with_address(i2c, LP5036_DEFAULT_ADDRESS)
    }

    /// Create a new LED driver instance with a custom I2C address and initialize the device
    pub fn new_with_address(i2c: &'a mut T, device_address: u8) -> Result<Self, LedDriverError> {
        let mut driver = Self {
            i2c,
            device_address,
        };
        driver.initialize()?;
        Ok(driver)
    }

    /// Initialize the LED driver by resetting and configuring the device
    fn initialize(&mut self) -> Result<(), LedDriverError> {
        // Reset the device
        self.reset()?;
        
        // Configure the device
        self.i2c
            .write_byte_to_register(
                self.device_address,
                DEVICE_CONFIG_0,
                DEVICE_CONFIG_0_SETTINGS,
            )
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Reset the LED driver device
    pub fn reset(&mut self) -> Result<(), LedDriverError> {
        self.i2c
            .write_byte_to_register(self.device_address, RESET, RESET_VALUE)
            .map_err(LedDriverError::I2cError)?;
        Ok(())
    }

    /// Set the brightness (0-255) of a specific LED channel
    pub fn set_channel_brightness(
        &mut self,
        channel: LedChannel,
        brightness: u8,
    ) -> Result<(), LedDriverError> {
        let channel_index = channel.index();
        if channel_index >= 36 {
            return Err(LedDriverError::InvalidChannel);
        }

        let register = LED_BRIGHTNESS_BASE + (channel_index / 3);
        
        self.i2c
            .write_byte_to_register(
                self.device_address,
                register,
                brightness,
            )
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Set the brightness for multiple channels at once
    pub fn set_multiple_brightness(
        &mut self,
        channels: &[(LedChannel, u8)],
    ) -> Result<(), LedDriverError> {
        for (channel, brightness) in channels {
            self.set_channel_brightness(*channel, *brightness)?;
        }
        Ok(())
    }

    /// Set the PWM value for a specific bank (0-255)
    pub fn set_bank_pwm(&mut self, bank: LedBank, pwm: u8) -> Result<(), LedDriverError> {
        let register = match bank {
            LedBank::BankA => BANK_A_PWM,
            LedBank::BankB => BANK_B_PWM,
            LedBank::BankC => BANK_C_PWM,
        };

        self.i2c
            .write_byte_to_register(self.device_address, register, pwm)
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Enable a specific LED channel
    pub fn enable_channel(&mut self, channel: LedChannel) -> Result<(), LedDriverError> {
        let channel_index = channel.index();
        if channel_index >= 36 {
            return Err(LedDriverError::InvalidChannel);
        }

        // LED configuration registers 0-2 control which of the 36 channels are enabled
        let config_register = LED_CONFIG_0 + (channel_index / 8) as u8;
        let bit_position = channel_index % 8;

        let current_config = self
            .i2c
            .read_byte(self.device_address, config_register)
            .ok_or(LedDriverError::I2cError(I2cError::Unknown))?;

        let new_config = current_config | (1 << bit_position);

        self.i2c
            .write_byte_to_register(self.device_address, config_register, new_config)
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Disable a specific LED channel
    pub fn disable_channel(&mut self, channel: LedChannel) -> Result<(), LedDriverError> {
        let channel_index = channel.index();
        if channel_index >= 36 {
            return Err(LedDriverError::InvalidChannel);
        }

        let config_register = LED_CONFIG_0 + (channel_index / 8) as u8;
        let bit_position = channel_index % 8;

        let current_config = self
            .i2c
            .read_byte(self.device_address, config_register)
            .ok_or(LedDriverError::I2cError(I2cError::Unknown))?;

        let new_config = current_config & !(1 << bit_position);

        self.i2c
            .write_byte_to_register(self.device_address, config_register, new_config)
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Enable multiple LED channels at once
    pub fn enable_channels(&mut self, channels: &[LedChannel]) -> Result<(), LedDriverError> {
        for channel in channels {
            self.enable_channel(*channel)?;
        }
        Ok(())
    }

    /// Disable multiple LED channels at once
    pub fn disable_channels(&mut self, channels: &[LedChannel]) -> Result<(), LedDriverError> {
        for channel in channels {
            self.disable_channel(*channel)?;
        }
        Ok(())
    }

    /// Set the color for a specific bank (0-255)
    pub fn set_bank_color(&mut self, bank: LedBank, color: u8) -> Result<(), LedDriverError> {
        let register = match bank {
            LedBank::BankA => BANK_A_COLOR,
            LedBank::BankB => BANK_B_COLOR,
            LedBank::BankC => BANK_C_COLOR,
        };

        self.i2c
            .write_byte_to_register(self.device_address, register, color)
            .map_err(LedDriverError::I2cError)?;
        
        Ok(())
    }

    /// Turn on an LED channel at full brightness
    pub fn turn_on(&mut self, channel: LedChannel) -> Result<(), LedDriverError> {
        self.enable_channel(channel)?;
        self.set_channel_brightness(channel, 255)?;
        Ok(())
    }

    /// Turn off an LED channel
    pub fn turn_off(&mut self, channel: LedChannel) -> Result<(), LedDriverError> {
        self.disable_channel(channel)?;
        self.set_channel_brightness(channel, 0)?;
        Ok(())
    }

    /// Turn on multiple LED channels at full brightness
    pub fn turn_on_multiple(&mut self, channels: &[LedChannel]) -> Result<(), LedDriverError> {
        for channel in channels {
            self.turn_on(*channel)?;
        }
        Ok(())
    }

    /// Turn off multiple LED channels
    pub fn turn_off_multiple(&mut self, channels: &[LedChannel]) -> Result<(), LedDriverError> {
        for channel in channels {
            self.turn_off(*channel)?;
        }
        Ok(())
    }
}


