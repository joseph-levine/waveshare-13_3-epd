use crate::constants::{command_code::CommandCode, known_pin::KnownPin};
use rppal::gpio::Level::{High, Low};
use rppal::gpio::Mode::{Input, Output};
use rppal::gpio::{Error as GpioError, Gpio, Level, OutputPin};
use rppal::spi::{Error as SpiError, Spi};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use tracing::info;

enum EpdError {
    Spi(SpiError),
    Gpio(GpioError),
}

impl From<SpiError> for EpdError {
    fn from(err: SpiError) -> EpdError {
        EpdError::Spi(err)
    }
}
impl From<GpioError> for EpdError {
    fn from(err: GpioError) -> EpdError {
        EpdError::Gpio(err)
    }
}

#[derive(Debug, Copy, Clone)]
enum SendMode {
    Command,
    Data,
}

impl From<SendMode> for Level {
    fn from(mode: SendMode) -> Self {
        match mode {
            SendMode::Command => Low,
            SendMode::Data => High,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EpdController {
    pub gpio: Gpio,
    pub spi: Spi,
}
impl EpdController {
    pub fn new(gpio: Gpio, spi: Spi) -> Result<EpdController, EpdError> {
        let mut this = EpdController { gpio, spi };
        this.output_pin(KnownPin::SerialClockPin)?.write(Low);
        this.output_pin(KnownPin::SerialDataPin)?.write(Low);
        this.output_pin(KnownPin::SerialSelectMainPin)?.write(Low);
        this.output_pin(KnownPin::SerialSelectPeriPin)?.write(Low);
        this.output_pin(KnownPin::DataCommandPin)?.write(Low);
        this.output_pin(KnownPin::ResetPin)?.write(Low);
        // this.output_pin(KnownPin::BusyPin)?.write(Low);
        this.output_pin(KnownPin::PowerPin)?.write(High);
        Ok(this)
    }

    pub fn output_pin(&self, known_pin: KnownPin) -> Result<OutputPin, GpioError> {
        Ok(self.gpio.get(known_pin.into())?.into_output())
    }

    pub fn send_command(
        &mut self,
        command_code: CommandCode,
        main_only: bool,
    ) -> Result<(), EpdError> {
        let mut main = self.output_pin(KnownPin::SerialSelectMainPin)?;
        let mut peri = self.output_pin(KnownPin::SerialSelectPeriPin)?;
        main.write(SendMode::Command.into());
        if !main_only {
            peri.write(SendMode::Command.into());
        }
        self.spi.write(command_code.cmd())?;
        if let Some(data) = command_code.data() {
            self.spi.write(data)?;
        }
        main.write(SendMode::Data.into());
        peri.write(SendMode::Data.into());
        Ok(())
    }

    pub fn reset(self) -> Result<(), EpdError> {
        let mut reset_pin = self.gpio.get(KnownPin::ResetPin.into())?.into_output();
        for l in [High, Low, High, Low, High] {
            reset_pin.write(l);
            sleep(Duration::from_millis(30));
        }
        Ok(())
    }

    pub fn wait_for_not_busy(self) -> Result<(), EpdError> {
        let busy_pin = self.gpio.get(KnownPin::BusyPin.into())?;
        while busy_pin.read() == Low {
            sleep(Duration::from_millis(5))
        }
        Ok(())
    }

    pub fn turn_display_on(&mut self) -> Result<(), EpdError> {
        info!("Write PON");
        self.send_command(CommandCode::Pon, false)?;
        self.wait_for_not_busy()?;

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.send_command(CommandCode::Drf, false)?;
        self.wait_for_not_busy()?;

        info!("Write POF");
        self.send_command(CommandCode::Pof, false)?;

        info!("Display Done");
        Ok(())
    }

    fn init(&mut self) -> Result<(), EpdError> {
        info!("EPD init...");
        self.reset()?;
        self.wait_for_not_busy()?;

        let boot_sequence = [
            (CommandCode::AnTm, true),
            (CommandCode::Cmd66, false),
            (CommandCode::Psr, false),
            (CommandCode::Cdi, false),
            (CommandCode::Tcon, false),
            (CommandCode::Agid, false),
            (CommandCode::Pws, false),
            (CommandCode::Ccset, false),
            (CommandCode::Tres, false),
            (CommandCode::Pwr, true),
            (CommandCode::EnBuf, true),
            (CommandCode::BtstP, true),
            (CommandCode::BoostVddpEn, true),
            (CommandCode::BtstN, true),
            (CommandCode::BuckBoostVddn, true),
            (CommandCode::TftVcomPower, true),
        ];
        for (command, main_only) in boot_sequence {
            self.send_command(command, main_only)?;
        }
        Ok(())
    }

    fn sleep_display(&mut self) -> Result<(), EpdError> {
        self.send_command(CommandCode::DeepSleep, false)?;
        sleep(Duration::from_secs(2));
        Ok(())
    }

}

impl Drop for EpdController {
    fn drop(&mut self) {
        if let Some(m) = self.output_pin(KnownPin::SerialSelectMainPin) {
            m.write(Low);
        };
        if let Some(p) = self.output_pin(KnownPin::SerialSelectPeriPin) {
            p.write(Low);
        }
        if let Some(dc) = self.output_pin(KnownPin::DataCommandPin) {
            dc.write(Low);
        }
        if let Some(r) = self.output_pin(KnownPin::ResetPin) {
            r.write(Low);
        }
        if let Some(pow) = self.output_pin(KnownPin::PowerPin) {
            pow.write(Low);
        }
    }
}

