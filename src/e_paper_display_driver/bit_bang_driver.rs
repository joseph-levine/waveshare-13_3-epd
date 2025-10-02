use crate::display_constants::{DISPLAY_BYTES_PER_CHIP, HALF_WIDTH, HEIGHT, WIDTH};
use crate::e_paper_display_driver::gpio_pin::Level::{High, Low};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use embedded_time::duration::Extensions;
use linux_embedded_hal::{CountDown, SysTimer};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::ops::Add;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use sysfs_gpio::{Direction, Error as GpioError, Pin};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum EpdError {
    #[error(transparent)]
    Io(#[from] IoError),
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
pub struct EPaperDisplayBBDriver {
    data_pin: Pin,
    chip_select_main_pin: Pin,
    chip_select_peri_pin: Pin,
    clock_pin: Pin,
    data_or_cmd_pin: Pin,
    reset_pin: Pin,
    busy_pin: Pin,
    power_pin: Pin,
    selected_chip: SelectedChip,
    send_mode: SendMode,
    timer: SysTimer,
}

impl EPaperDisplayBBDriver {
    pub fn new() -> Result<EPaperDisplayBBDriver, EpdError> {
        let clock_pin = GpioPin::SerialClockPin.pin(Low, None)?;

        let chip_select_main_pin = GpioPin::SerialSelectMainPin.pin(Low, None)?;

        let chip_select_peri_pin = GpioPin::SerialSelectPeriPin.pin(Low, None)?;

        let data_or_cmd_pin = GpioPin::DataCommandPin.pin(Low, None)?;

        let busy_pin = GpioPin::BusyPin.pin(Low, Some(Direction::In))?;

        let reset_pin = GpioPin::ResetPin.pin(Low, None)?;

        let power_pin = GpioPin::PowerPin.pin(High, None)?;

        let data_pin = GpioPin::SerialDataPin.pin(Low, None)?;

        Ok(EPaperDisplayBBDriver {
            data_pin,
            clock_pin,
            data_or_cmd_pin,
            chip_select_main_pin,
            chip_select_peri_pin,
            reset_pin,
            busy_pin,
            power_pin,
            send_mode: SendMode::Command,
            selected_chip: SelectedChip::Both,
            timer: SysTimer::new(),
        })
    }

    fn spi_write(&mut self, bytes: &[u8]) -> Result<(), EpdError> {
        self.timer.start(1.microseconds()).expect("Infallible");
        for byte in bytes {
            let mut b = byte.clone();
            for i in 0..u8::MAX {
                self.clock_pin.set_value(Low as u8)?;
                self.data_pin
                    .set_value(if b & 0x80 == 0x80 { High } else { Low } as u8)?;
                b = b << 1;
                self.timer.wait()?;
                self.clock_pin.set_value(High as u8)?;
                self.timer.wait()?;
            }
        }
        Ok(())
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

    fn send_command(
        &mut self,
        command_code: CommandCode,
        selected_chip: SelectedChip,
    ) -> Result<(), EpdError> {
        self.select_chip(selected_chip)?;
        self.set_send_mode(SendMode::Command)?;
        self.spi_write(&[command_code.cmd()])?;
        if let Some(data) = command_code.data() {
            self.set_send_mode(SendMode::Data)?;
            self.spi_write(data)?;
        }
        Ok(())
    }

    fn wait_for_not_busy(&self) -> Result<(), EpdError> {
        while self.busy_pin.get_value()? == (Low as u8) {
            sleep(Duration::from_millis(10));
        }
        sleep(Duration::from_millis(20));
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

    pub fn boot_display(&mut self) -> Result<(), EpdError> {
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
        for (command, chip) in boot_sequence {
            self.send_command(command, chip)?;
        }
        Ok(())
    }

    pub fn sleep_display(&mut self) -> Result<(), EpdError> {
        self.send_command(CommandCode::DeepSleep, SelectedChip::Both)?;
        sleep(Duration::from_secs(2));
        Ok(())
    }

    fn reset(&self) -> Result<(), EpdError> {
        for l in [High, Low, High, Low, High] {
            self.reset_pin.set_value(l as u8)?;
            sleep(Duration::from_millis(30));
        }
        Ok(())
    }
}

impl EPaperDisplayBBDriver {
    pub fn clear_screen(&mut self) -> Result<(), EpdError> {
        let zeros: &[u8; DISPLAY_BYTES_PER_CHIP] = &[0u8; DISPLAY_BYTES_PER_CHIP];
        self.spi_write(zeros)?;
        Ok(())
    }

    pub fn send_image(&mut self, image: &[u8]) -> Result<(), EpdError> {
        assert_eq!(image.len(), HEIGHT * WIDTH / 2);
        let mut top: [u8; DISPLAY_BYTES_PER_CHIP] = [0u8; DISPLAY_BYTES_PER_CHIP];
        let mut bottom: [u8; DISPLAY_BYTES_PER_CHIP] = [0u8; DISPLAY_BYTES_PER_CHIP];
        for (k, v) in image.iter().enumerate() {
            let column = k % WIDTH;
            let row = k / WIDTH;
            if column < HALF_WIDTH {
                top[row * HALF_WIDTH + column] = *v;
            } else {
                bottom[row * HALF_WIDTH + (column - HALF_WIDTH)] = *v;
            }
        }

        self.send_command(CommandCode::Dtm, SelectedChip::Main)?;
        self.spi_write(top.as_ref())?;
        self.send_command(CommandCode::Dtm, SelectedChip::Peri)?;
        self.spi_write(bottom.as_ref())?;
        Ok(())
    }
}

impl Drop for EPaperDisplayBBDriver {
    fn drop(&mut self) {
        // we're going to ignore errors here...
        let _ = self.chip_select_main_pin.set_value(Low as u8);
        let _ = self.chip_select_peri_pin.set_value(Low as u8);
        let _ = self.data_or_cmd_pin.set_value(Low as u8);
        let _ = self.reset_pin.set_value(Low as u8);
        let _ = self.power_pin.set_value(Low as u8);
    }
}
