use crate::constants::command_code::CommandCode;

#[derive(Debug)]
#[repr(u8)]
pub enum KnownPin {
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

impl From<KnownPin> for u8 {
    fn from(known_pin: KnownPin) -> Self {
        known_pin as u8
    }
}
