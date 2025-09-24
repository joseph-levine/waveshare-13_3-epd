use crate::constants::known_pin::Level::{High, Low};
use crate::constants::{command_code::CommandCode, known_pin::KnownPin};
use embedded_hal::spi::SpiDevice;
use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::sysfs_gpio::{Direction, Error as GpioError, Pin};
use linux_embedded_hal::{SPIError, SpidevDevice};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum EpdError {
    #[error(transparent)]
    Spi(#[from] SPIError),
    #[error(transparent)]
    Io(#[from]io::Error),
    #[error(transparent)]
    Gpio(#[from] GpioError),
}


#[derive(Debug, Copy, Clone, PartialEq)]
enum SendMode {
    Command,
    Data,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SelectedChip {
    Main,
    Peri,
    Both,
}

#[derive(Debug)]
pub struct EpdDevice<SPI>
where
    SPI: SpiDevice,
{
    spi: SPI,
    chip_select_main_pin: Pin,
    chip_select_peri_pin: Pin,
    clock_pin: Pin,
    data_or_cmd_pin: Pin,
    // data_or_cmd_pin: Pin,
    reset_pin: Pin,
    busy_pin: Pin,
    power_pin: Pin,
    selected_chip: SelectedChip,
    send_mode: SendMode,
}

impl EpdDevice<SpidevDevice> {
    pub fn new() -> Result<EpdDevice<SpidevDevice>, EpdError> {
        let mut spi = SpidevDevice::open("/dev/spidev0.0")?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(32_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;

        let clock_pin = KnownPin::SerialClockPin.pin(Low, None)?;

        let chip_select_main_pin = KnownPin::SerialSelectMainPin.pin(Low, None)?;

        let chip_select_peri_pin = KnownPin::SerialSelectPeriPin.pin(Low, None)?;

        let data_or_cmd_pin = KnownPin::DataCommandPin.pin(Low, None)?;

        let busy_pin = KnownPin::BusyPin.pin(Low, Some(Direction::Out))?;

        let reset_pin = KnownPin::ResetPin.pin(Low, None)?;

        let power_pin = KnownPin::PowerPin.pin(High, None)?;

        Ok(EpdDevice {
            spi,
            clock_pin,
            data_or_cmd_pin,
            chip_select_main_pin,
            chip_select_peri_pin,
            reset_pin,
            busy_pin,
            power_pin,
            send_mode: SendMode::Command,
            selected_chip: SelectedChip::Both,
        })
    }

    fn select_chip(&mut self, selected_chip: SelectedChip) -> Result<(), EpdError> {
        if selected_chip == self.selected_chip {
            return Ok(());
        }
        match (self.selected_chip, selected_chip) {
            (SelectedChip::Both, SelectedChip::Main) => {
                self.chip_select_peri_pin.set_value(Low as u8)?;
            }
            (SelectedChip::Both, SelectedChip::Peri) => {
                self.chip_select_main_pin.set_value(Low as u8)?;
            }
            (SelectedChip::Main, SelectedChip::Both) => {
                self.chip_select_peri_pin.set_value(High as u8)?;
            }
            (SelectedChip::Main, SelectedChip::Peri) => {
                self.chip_select_main_pin.set_value(Low as u8)?;
                self.chip_select_peri_pin.set_value(High as u8)?;
            }
            (SelectedChip::Peri, SelectedChip::Both) => {
                self.chip_select_main_pin.set_value(High as u8)?;
            }
            (SelectedChip::Peri, SelectedChip::Main) => {
                self.chip_select_main_pin.set_value(High as u8)?;
                self.chip_select_peri_pin.set_value(Low as u8)?;
            }
            (_, _) => unreachable!(),
        }
        self.selected_chip = selected_chip;
        Ok(())
    }

    fn set_send_mode(&mut self, send_mode: SendMode) -> Result<(), EpdError> {
        if send_mode == self.send_mode {
            return Ok(());
        }
        match (self.send_mode, send_mode) {
            (SendMode::Command, SendMode::Data) => {
                self.data_or_cmd_pin.set_value(High as u8)?;
            }
            (SendMode::Data, SendMode::Command) => {
                self.data_or_cmd_pin.set_value(Low as u8)?;
            }
            (_, _) => unreachable!(),
        }
        self.send_mode = send_mode;
        Ok(())
    }

    pub fn send_command(
        &mut self,
        command_code: CommandCode,
        selected_chip: SelectedChip,
    ) -> Result<(), EpdError> {
        self.select_chip(selected_chip)?;
        self.set_send_mode(SendMode::Command)?;
        self.spi.write(&[command_code.cmd()])?;
        if let Some(data) = command_code.data() {
            self.set_send_mode(SendMode::Data)?;
            self.spi.write(data)?;
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), EpdError> {
        for l in [High, Low, High, Low, High] {
            self.reset_pin.set_value(l as u8)?;
            sleep(Duration::from_millis(30));
        }
        Ok(())
    }

    fn wait_for_not_busy(&self) -> Result<(), EpdError> {
        while self.busy_pin.get_value()? == (Low as u8) {
            sleep(Duration::from_millis(5))
        }
        Ok(())
    }

    pub fn turn_display_on(&mut self) -> Result<(), EpdError> {
        info!("Write PON");
        self.send_command(CommandCode::Pon, SelectedChip::Both)?;
        self.wait_for_not_busy()?;

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.send_command(CommandCode::Drf, SelectedChip::Both)?;
        self.wait_for_not_busy()?;

        info!("Write POF");
         self.send_command(CommandCode::Pof, SelectedChip::Both)?;

        info!("Display Done");
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), EpdError> {
        info!("EPD init...");
        self.reset()?;
        self.wait_for_not_busy()?;

        let boot_sequence = [
            (CommandCode::AnTm, SelectedChip::Main),
            (CommandCode::Cmd66, SelectedChip::Both),
            (CommandCode::Psr, SelectedChip::Both),
            (CommandCode::Cdi, SelectedChip::Both),
            (CommandCode::Tcon, SelectedChip::Both),
            (CommandCode::Agid, SelectedChip::Both),
            (CommandCode::Pws, SelectedChip::Both),
            (CommandCode::Ccset, SelectedChip::Both),
            (CommandCode::Tres, SelectedChip::Both),
            (CommandCode::Pwr, SelectedChip::Main),
            (CommandCode::EnBuf, SelectedChip::Main),
            (CommandCode::BtstP, SelectedChip::Main),
            (CommandCode::BoostVddpEn, SelectedChip::Main),
            (CommandCode::BtstN, SelectedChip::Main),
            (CommandCode::BuckBoostVddn, SelectedChip::Main),
            (CommandCode::TftVcomPower, SelectedChip::Main),
        ];
        for (command, main_only) in boot_sequence {
            self.send_command(command, main_only)?;
        }
        Ok(())
    }

    pub fn sleep_display(&mut self) -> Result<(), EpdError> {
        self.send_command(CommandCode::DeepSleep, SelectedChip::Both)?;
        sleep(Duration::from_secs(2));
        Ok(())
    }
}

impl<SPI> Drop for EpdDevice<SPI>
where
    SPI: SpiDevice,
{
    fn drop(&mut self) {
        // we're going to ignore errors here...
        let _ = self.chip_select_main_pin.set_value(Low as u8);
        let _ = self.chip_select_peri_pin.set_value(Low as u8);
        let _ = self.data_or_cmd_pin.set_value(Low as u8);
        let _ = self.reset_pin.set_value(Low as u8);
        let _ = self.power_pin.set_value(Low as u8);
    }
}
