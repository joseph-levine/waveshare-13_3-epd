use crate::display_constants::{DISPLAY_BYTES_PER_CHIP, HALF_WIDTH, HEIGHT, WIDTH};
// use crate::e_paper_display_driver::gpio_pin::Level::{High, Low};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use linux_embedded_hal::{CountDown, SysTimer};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::thread::sleep;
use std::time::Duration;
use linux_embedded_hal::nb::block;
use rppal::gpio::{Gpio, InputPin, OutputPin, Error as GpioError};
use rppal::gpio::Level::{High, Low};
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
    gpio: Gpio,
    data_pin: OutputPin,
    chip_select_main_pin: OutputPin,
    chip_select_peri_pin: OutputPin,
    clock_pin: OutputPin,
    data_or_cmd_pin: OutputPin,
    reset_pin: OutputPin,
    busy_pin: InputPin,
    power_pin: OutputPin,
    selected_chip: SelectedChip,
    send_mode: SendMode,
}

impl EPaperDisplayBBDriver {
    pub fn new() -> Result<EPaperDisplayBBDriver, EpdError> {
        let gpio = Gpio::new()?;

        let mut clock_pin = gpio.get(GpioPin::SerialClockPin as u8)?.into_output();
        clock_pin.set_low();

        let mut chip_select_main_pin = gpio.get(GpioPin::SerialSelectMainPin as u8)?.into_output();
        chip_select_main_pin.set_low();

        let mut chip_select_peri_pin = gpio.get(GpioPin::SerialSelectPeriPin as u8)?.into_output();
        chip_select_peri_pin.set_low();

        let mut data_or_cmd_pin = gpio.get(GpioPin::DataCommandPin as u8)?.into_output();
        data_or_cmd_pin.set_low();

        let busy_pin = gpio.get(GpioPin::BusyPin as u8)?.into_input();

        let mut reset_pin = gpio.get(GpioPin::ResetPin as u8)?.into_output();
        reset_pin.set_low();

        let mut power_pin = gpio.get(GpioPin::PowerPin as u8)?.into_output();
        power_pin.set_high();

        let mut data_pin = gpio.get(GpioPin::SerialDataPin as u8)?.into_output();
        data_pin.set_low();

        Ok(EPaperDisplayBBDriver {
            gpio,
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
        })
    }

    fn spi_write(&mut self, bytes: &[u8]) {
        // let mut timer = SysTimer::new();
        // timer.start(Duration::from_micros(1)).expect("Infallible");
        for byte in bytes {
            let mut b = byte.clone();
            for _i in 0..u8::MAX {
                self.clock_pin.write(Low);
                self.data_pin
                    .write(if b & 0x80 == 0x80 { High } else { Low } );
                b = b << 1;
                // sleep(Duration::from_micros(1));
                self.clock_pin.write(High);
                // sleep(Duration::from_micros(1));;
            }
        }
    }

    fn select_chip(&mut self, selected_chip: SelectedChip){
        if selected_chip == self.selected_chip {
            return ;
        }
        match (self.selected_chip, selected_chip) {
            (SelectedChip::Both, SelectedChip::Main) => {
                self.chip_select_peri_pin.write(Low);
            }
            (SelectedChip::Both, SelectedChip::Peri) => {
                self.chip_select_main_pin.write(Low);
            }
            (SelectedChip::Main, SelectedChip::Both) => {
                self.chip_select_peri_pin.write(High);
            }
            (SelectedChip::Main, SelectedChip::Peri) => {
                self.chip_select_main_pin.write(Low);
                self.chip_select_peri_pin.write(High);
            }
            (SelectedChip::Peri, SelectedChip::Both) => {
                self.chip_select_main_pin.write(High);
            }
            (SelectedChip::Peri, SelectedChip::Main) => {
                self.chip_select_main_pin.write(High);
                self.chip_select_peri_pin.write(Low);
            }
            (_, _) => unreachable!(),
        }
        self.selected_chip = selected_chip;
    }

    fn set_send_mode(&mut self, send_mode: SendMode) {
        if send_mode == self.send_mode {
            return ;
        }
        match (self.send_mode, send_mode) {
            (SendMode::Command, SendMode::Data) => {
                self.data_or_cmd_pin.write(High);
            }
            (SendMode::Data, SendMode::Command) => {
                self.data_or_cmd_pin.write(Low);
            }
            (_, _) => unreachable!(),
        }
        self.send_mode = send_mode;
    }

    fn send_command(
        &mut self,
        command_code: CommandCode,
        selected_chip: SelectedChip,
    )  {
        self.select_chip(selected_chip);
        self.set_send_mode(SendMode::Command);
        self.spi_write(&[command_code.cmd()]);
        if let Some(data) = command_code.data() {
            self.set_send_mode(SendMode::Data);
            self.spi_write(data);
        }
    }

    fn wait_for_not_busy(&self) {
        while self.busy_pin.read() == (Low) {
            sleep(Duration::from_millis(10));
        }
        sleep(Duration::from_millis(20));
    }

    pub fn turn_display_on(&mut self) {
        info!("Write PON");
        self.send_command(CommandCode::Pon, SelectedChip::Both);
        self.wait_for_not_busy();

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.send_command(CommandCode::Drf, SelectedChip::Both);
        self.wait_for_not_busy();

        info!("Write POF");
        self.send_command(CommandCode::Pof, SelectedChip::Both);

        info!("Display Done");
    }

    pub fn boot_display(&mut self) {
        info!("EPD init...");
        self.reset();
        self.wait_for_not_busy();

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
            self.send_command(command, chip);
        }
    }

    pub fn sleep_display(&mut self) {
        self.send_command(CommandCode::DeepSleep, SelectedChip::Both);
        sleep(Duration::from_secs(2));
    }

    fn reset(&mut self) {
        for l in [High, Low, High, Low, High] {
            self.reset_pin.write(l);
            sleep(Duration::from_millis(30));
        }
    }
}

impl EPaperDisplayBBDriver {
    pub fn clear_screen(&mut self) {
        let zeros: &[u8; DISPLAY_BYTES_PER_CHIP] = &[0u8; DISPLAY_BYTES_PER_CHIP];
        self.spi_write(zeros);
    }

    pub fn send_image(&mut self, image: &[u8])  {
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

        self.send_command(CommandCode::Dtm, SelectedChip::Main);
        self.spi_write(top.as_ref());
        self.send_command(CommandCode::Dtm, SelectedChip::Peri);
        self.spi_write(bottom.as_ref());
    }
}

impl Drop for EPaperDisplayBBDriver {
    fn drop(&mut self) {
        // we're going to ignore errors here...
        let _ = self.chip_select_main_pin.write(Low);
        let _ = self.chip_select_peri_pin.write(Low);
        let _ = self.data_or_cmd_pin.write(Low);
        let _ = self.reset_pin.write(Low);
        let _ = self.power_pin.write(Low);
    }
}
