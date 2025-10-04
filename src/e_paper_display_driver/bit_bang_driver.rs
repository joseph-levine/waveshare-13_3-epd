use crate::display_constants::{DISPLAY_BYTES_PER_CHIP, HALF_WIDTH, HEIGHT, WIDTH};
// use crate::e_paper_display_driver::gpio_pin::Level::{High, Low};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
// use linux_embedded_hal::nb::block;
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
enum SelectedChip {
    Main,
    Peri,
    Both,
    Neither,
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
}

fn spin_sleep(duration: Duration) -> u64 {
    let mut loops = 0u64;
    let start = Instant::now();
    while Instant::now().duration_since(start) < duration {
        loops += 1;
    }
    loops
}

impl EPaperDisplayBBDriver {
    pub fn new() -> Result<EPaperDisplayBBDriver, EpdError> {
        let gpio = Gpio::new()?;

        let mut clock_pin = gpio.get(GpioPin::SerialClockPin as u8)?.into_output();
        let mut data_pin = gpio.get(GpioPin::SerialDataPin as u8)?.into_output();
        let mut chip_select_main_pin = gpio.get(GpioPin::SerialSelectMainPin as u8)?.into_output();
        let mut chip_select_peri_pin = gpio.get(GpioPin::SerialSelectPeriPin as u8)?.into_output();
        let mut data_or_cmd_pin = gpio.get(GpioPin::DataCommandPin as u8)?.into_output();
        let mut reset_pin = gpio.get(GpioPin::ResetPin as u8)?.into_output();
        let busy_pin = gpio.get(GpioPin::BusyPin as u8)?.into_input();
        let mut power_pin = gpio.get(GpioPin::PowerPin as u8)?.into_output();

        clock_pin.set_low();
        data_pin.set_low();
        chip_select_main_pin.set_low();
        chip_select_peri_pin.set_low();
        data_or_cmd_pin.set_low();
        reset_pin.set_low();
        power_pin.set_high();

        let mut this = EPaperDisplayBBDriver {
            gpio,
            data_pin,
            clock_pin,
            data_or_cmd_pin,
            chip_select_main_pin,
            chip_select_peri_pin,
            reset_pin,
            busy_pin,
            power_pin,
            selected_chip: SelectedChip::Both,
        };

        this.reset();
        this.wait_for_not_busy();

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
            this.send_command(command, chip);
        }

        Ok(this)
    }

    fn spi_write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let mut b = byte.clone();
            for _i in 0..u8::MAX {
                let _ = spin_sleep(Duration::from_micros(1));
                self.clock_pin.write(Low);
                self.data_pin
                    .write(if b & 0x80 == 0x80 { High } else { Low } );
                b = b << 1;
                let _ = spin_sleep(Duration::from_micros(1));
                self.clock_pin.write(High);
            }
        }
        info!("spi written");
    }

    fn select_chip(&mut self, new_selection: SelectedChip){
        if new_selection == self.selected_chip {
            return ;
        }
        // low is selected
        // should set main on?
        // if new in (both, main) and selected not in (both, main)
        self.chip_select_main_pin.write(if new_selection == SelectedChip::Main || new_selection == SelectedChip::Both { Low } else { High });
        self.chip_select_peri_pin.write(if new_selection == SelectedChip::Main || new_selection == SelectedChip::Peri { Low } else { High });
        self.selected_chip = new_selection;
    }

    fn send_command(
        &mut self,
        command_code: CommandCode,
        selected_chip: SelectedChip,
    )  {
        self.select_chip(selected_chip);
        let mut full_cmd = vec![command_code.cmd()];
        if let Some(data) = command_code.data() {
            full_cmd.extend_from_slice(data);
        }
        self.spi_write(full_cmd.as_slice());
        self.select_chip(SelectedChip::Neither);
    }

    fn wait_for_not_busy(&self) {
        info!("waiting for not busy");
        while self.busy_pin.read() == Low {
            sleep(Duration::from_millis(10));
        }
        sleep(Duration::from_millis(20));
    }

    fn turn_display_on(&mut self) {
        info!("Write PON");
        self.send_command(CommandCode::PowerOn, SelectedChip::Both);
        self.wait_for_not_busy();

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.send_command(CommandCode::Drf, SelectedChip::Both);
        self.wait_for_not_busy();

        info!("Write POF");
        self.send_command(CommandCode::Pof, SelectedChip::Both);

        info!("Display On");
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
        self.select_chip(SelectedChip::Main);
        self.spi_write(zeros);
        self.select_chip(SelectedChip::Neither);
        self.select_chip(SelectedChip::Peri);
        self.spi_write(zeros);
        self.select_chip(SelectedChip::Neither);
        self.turn_display_on();
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

        self.select_chip(SelectedChip::Main);
        self.spi_write(&[CommandCode::Dtm.cmd()]);
        self.spi_write(top.as_ref());
        self.select_chip(SelectedChip::Neither);
        self.select_chip(SelectedChip::Peri);
        self.spi_write(&[CommandCode::Dtm.cmd()]);
        self.spi_write(bottom.as_ref());
        self.select_chip(SelectedChip::Neither);
        self.turn_display_on();
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
