use crate::e_paper_display_driver::bcm2835::{
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_INPT as FSEL_INPUT,
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as FSEL_OUTPUT, bcm2835_gpio_fsel,
    bcm2835_gpio_lev, bcm2835_gpio_write,
};

#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Level {
    Low = 0x00,
    High = 0x01,
}

impl From<Level> for u8 {
    fn from(value: Level) -> Self {
        value as u8
    }
}

impl From<u8> for Level {
    fn from(value: u8) -> Self {
        let try_cast = match value {
            0x00 => Ok(Self::Low),
            0x01 => Ok(Self::High),
            _ => Err(()),
        };
        try_cast.expect("invalid level")
    }
}

pub trait GpioReadWrite {
    fn set_all_modes();
    fn set_mode(&self);
    fn write(&self, level: Level);
    fn read(&self) -> Level;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GpioPin {
    /// serial clock
    SerialClockPin = 11,

    /// Serial data signal
    SerialDataPin = 10,

    /// Serial communication chip select (Main)
    SerialSelectMainPin = 8,
    /// Serial communication chip select (Peripheral)
    SerialSelectPeriPin = 7,

    /// Data/command control signal, write command (Command) when the level is low; write data (Data/parameter) when the level is high
    DataCommandPin = 25,
    /// Reset
    ResetPin = 17,
    /// This pin indicates the driver status.
    BusyPin = 24,
    PowerPin = 18,
}

impl From<GpioPin> for u8 {
    fn from(value: GpioPin) -> Self {
        value as u8
    }
}

impl GpioReadWrite for GpioPin {
    fn set_all_modes() {
        for pin in &[
            Self::SerialClockPin,
            Self::SerialDataPin,
            Self::SerialSelectMainPin,
            Self::SerialSelectPeriPin,
            Self::DataCommandPin,
            Self::ResetPin,
            Self::BusyPin,
            Self::PowerPin,
        ] {
            pin.set_mode();
        }
    }

    fn set_mode(&self) {
        let pin: u8 = (*self).into();

        let direction = match self {
            Self::SerialClockPin => FSEL_OUTPUT,
            Self::SerialDataPin => FSEL_OUTPUT,
            Self::SerialSelectMainPin => FSEL_OUTPUT,
            Self::SerialSelectPeriPin => FSEL_OUTPUT,
            Self::DataCommandPin => FSEL_OUTPUT,
            Self::ResetPin => FSEL_OUTPUT,
            Self::BusyPin => FSEL_INPUT,
            Self::PowerPin => FSEL_OUTPUT,
        } as u8;

        /// SAFETY: all these specific cases should be safe
        unsafe {
            bcm2835_gpio_fsel(pin, direction)
        }
    }

    fn write(&self, level: Level) {
        // can't write to busy pin
        assert_ne!(*self, Self::BusyPin);
        let pin: u8 = (*self).into();
        let level_u8: u8 = level.into();
        /// SAFETY: if the pin hasn't been initialized this will probably be undefined behavior.
        /// For this specific display, only the BusyPin should fail to write, and that's handled in the above assertion.
        unsafe {
            bcm2835_gpio_write(pin, level_u8);
        }
    }

    fn read(&self) -> Level {
        // the display only supports reading from the busy pin
        assert_eq!(*self, Self::BusyPin);
        let pin = (*self).into();
        let mut level_u8: u8 ;
        /// SAFETY: if the pin hasn't been initialized this will probably be undefined behavior.
        /// For this specific display, only the BusyPin should be able to read, and that's handled in the above assertion.
        unsafe {
            level_u8 = bcm2835_gpio_lev(pin);
        }
        level_u8.into()
    }
}

// impl From<GpioPin> for u64 {
//     fn from(known_pin: GpioPin) -> Self {
//         known_pin as u64
//     }
// }
//
