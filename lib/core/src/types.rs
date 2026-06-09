#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DigitalSignal {
    High,
    Low,
}

impl DigitalSignal {
    pub fn from_bool(signal: bool) -> DigitalSignal {
        if signal {
            DigitalSignal::High
        } else {
            DigitalSignal::Low
        }
    }
}

impl From<DigitalSignal> for bool {
    fn from(signal: DigitalSignal) -> bool {
        match signal {
            DigitalSignal::High => true,
            DigitalSignal::Low => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Airgap(pub u16); // micrometers

// Calculate absolute distance between two airgaps
impl Airgap {
    pub fn distance_to(&self, other: Airgap) -> u16 {
        self.0.abs_diff(other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Current(pub u16); // milliamps

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Voltage(pub u16); // centivolts

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Pressure(pub u16); // bar

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Velocity(pub u16); // km/h

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Temperature(pub u8); // celsius

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub struct Force(pub u16); // newtons

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bool() {
        assert_eq!(DigitalSignal::from_bool(true), DigitalSignal::High);
        assert_eq!(DigitalSignal::from_bool(false), DigitalSignal::Low);
    }
}
