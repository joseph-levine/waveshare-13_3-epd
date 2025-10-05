// use crate::e_paper_display_driver::bcm_peripherals::Level::{High,Low};
use crate::display_constants::{DISPLAY_BYTES_PER_CHIP, HALF_WIDTH, HEIGHT, WIDTH};
// use crate::e_paper_display_driver::gpio_pin::Level::{High, Low};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
// use linux_embedded_hal::nb::block;
use rppal::gpio::Level::{High, Low};
use rppal::gpio::{Error as GpioError, Gpio, InputPin, OutputPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::thread::sleep;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, info};

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

fn spin_sleep() {
    let mut loops = 10;
    while loops > 0 {
        loops -= 1;
    }
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

        debug!("{:?} low", GpioPin::SerialClockPin as u8);
        clock_pin.set_low();
        debug!("{:?} low", GpioPin::SerialDataPin as u8);
        data_pin.set_low();
        debug!("{:?} low", GpioPin::SerialSelectMainPin as u8);
        chip_select_main_pin.set_low();
        debug!("{:?} low", GpioPin::SerialSelectPeriPin as u8);
        chip_select_peri_pin.set_low();
        debug!("{:?} low", GpioPin::DataCommandPin as u8);
        data_or_cmd_pin.set_low();
        debug!("{:?} low", GpioPin::BusyPin as u8);
        reset_pin.set_low();
        debug!("{:?} high", GpioPin::PowerPin as u8);
        power_pin.set_high();

        let mut this = EPaperDisplayBBDriver {
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
        if bytes.len() > 32 {
            debug!("Spi write to {} bytes", bytes.len());
        } else {
            debug!("Spi write {:02X?}", bytes);
        }
        for byte in bytes {
            let mut b = byte.clone();
            for _i in 0..8 {
                spin_sleep();
                self.clock_pin.write(Low);
                self.data_pin
                    .write(if b & 0x80 == 0x80 { High } else { Low });
                b = b << 1;
                spin_sleep();
                self.clock_pin.write(High);
            }
        }
    }

    fn select_chip(&mut self, new_selection: SelectedChip) {
        if new_selection == self.selected_chip {
            return;
        }
        let main_selected = [SelectedChip::Main, SelectedChip::Both].contains(&self.selected_chip);
        let select_main = [SelectedChip::Main, SelectedChip::Both].contains(&new_selection);
        let peri_selected = [SelectedChip::Peri, SelectedChip::Both].contains(&self.selected_chip);
        let select_peri = [SelectedChip::Peri, SelectedChip::Both].contains(&new_selection);
        if main_selected != select_main {
            debug!("digital_write pin: 8, value: {}", if select_main {"0"} else {"1"} );
            self.chip_select_main_pin
                .write(if select_main { Low } else { High });
        }
        if peri_selected != select_peri {
            debug!("digital_write pin: 7, value: {}", if select_peri {"0"} else {"1"} );
            self.chip_select_peri_pin
                .write(if select_peri { Low } else { High });
        }
        self.selected_chip = new_selection;
    }

    fn send_command(
        &mut self,
        command_code: CommandCode,
        selected_chip: SelectedChip,
    ) {
        self.select_chip(selected_chip);
        let mut full_cmd = vec![command_code.cmd()];
        if let Some(data) = command_code.data() {
            full_cmd.extend_from_slice(data);
        }
        self.spi_write(full_cmd.as_slice());
        self.select_chip(SelectedChip::Neither);
    }

    fn wait_for_not_busy(&self) {
        sleep(Duration::from_millis(20));
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
            debug!("Reset: {:?}", &l);
            self.reset_pin.write(l);
            sleep(Duration::from_millis(30));
        }
    }
}

impl EPaperDisplayBBDriver {
    pub fn clear_screen(&mut self) {
        let zeros: &[u8; DISPLAY_BYTES_PER_CHIP] = &[0u8; DISPLAY_BYTES_PER_CHIP];
        self.select_chip(SelectedChip::Main);
        self.spi_write(&[CommandCode::Dtm.cmd()]);
        self.spi_write(zeros);
        self.select_chip(SelectedChip::Neither);
        self.select_chip(SelectedChip::Peri);
        self.spi_write(&[CommandCode::Dtm.cmd()]);
        self.spi_write(zeros);
        self.select_chip(SelectedChip::Neither);
        self.turn_display_on();
    }

    pub fn send_image(&mut self, image: &[u8]) {
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
