#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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

// impl From<GpioPin> for u64 {
//     fn from(known_pin: GpioPin) -> Self {
//         known_pin as u64
//     }
// }
//
