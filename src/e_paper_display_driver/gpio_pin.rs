use std::io;
use sysfs_gpio::{Direction, Pin};

#[derive(Debug)]
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

#[derive(Debug)]
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

impl From<GpioPin> for u64 {
    fn from(known_pin: GpioPin) -> Self {
        known_pin as u64
    }
}

impl GpioPin {
    pub fn pin(self, initial_value: Level, direction: Option<Direction>) -> Result<Pin,io::Error> {
        let p = Pin::new(self.into());
        p.export()?;
        if let Some(d) = direction {
            p.set_direction(d)?;
        }
        p.set_value(initial_value.into())?;
        Ok(p)
    }
}
