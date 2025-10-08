use crate::display_constants::{DISPLAY_BYTES_PER_CHIP, HALF_WIDTH, HEIGHT, WIDTH};
use crate::e_paper_display_driver::bcm2835::{bcm2835_init, bcm2835_spi_transfer, bcm2835_spi_transfernb};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::os::raw::{c_char, c_int};
use std::thread::sleep;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::info;
use crate::e_paper_display_driver::gpio_pin::{GpioReadWrite, Level};

#[derive(Debug)]
pub enum EpdError {
    // #[error(transparent)]
    Io(IoError),
    #[error("broadcom init error")]BcmInitError
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SelectedChip {
    Main,
    Peri,
    Both,
    Neither,
}

#[derive(Debug)]
pub struct EPaperDisplayBcmDriver {
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

impl EPaperDisplayBcmDriver {
    pub fn new() -> EPaperDisplayBcmDriver {
        let init_status: c_int;
        /// Safety: Should return status instead of crashing...
        unsafe {
            init_status = bcm2835_init();
        }
        if init_status != 0 {
            panic!("BCM2835 initialization failed");
        }

        GpioPin::set_all_modes();

        let clock_pin = GpioPin::SerialClockPin;
        let data_pin = GpioPin::SerialDataPin;
        let chip_select_main_pin = GpioPin::SerialSelectMainPin;
        let chip_select_peri_pin = GpioPin::SerialSelectPeriPin;
        let data_or_cmd_pin = GpioPin::DataCommandPin;
        let reset_pin = GpioPin::ResetPin;
        let power_pin = GpioPin::PowerPin;

        clock_pin.write(Level::Low);
        data_pin.write(Level::Low);
        chip_select_main_pin.write(Level::Low);
        chip_select_peri_pin.write(Level::Low);
        data_or_cmd_pin.write(Level::Low);
        reset_pin.write(Level::Low);
        power_pin.write(Level::High);

        let mut this = EPaperDisplayBcmDriver {
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

        this
    }

    fn spi_write_byte(&mut self, byte: u8) {
        /// SAFETY: If SPI hasn't been set up correctly
        unsafe {
            bcm2835_spi_transfer(byte);
        }
    }

    fn spi_write(&mut self, bytes: &[u8]) {
        let length = bytes.len();
        assert!(length < u32::MAX as usize);

        let mut v_bytes = Vec::from(bytes);
        let mut c_send_chars = v_bytes.as_ptr() as *mut c_char;
        let mut received = Vec::with_capacity(length);
        let mut c_received_chars = received.as_mut_ptr() as *mut c_char;

        /// SAFETY: If SPI hasn't been set up correctly
        unsafe {
            bcm2835_spi_transfernb(c_send_chars, c_received_chars, length as u32);
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
            GpioPin::SerialSelectMainPin
                .write(if select_main { Level::Low } else { Level::High });
        }
        if peri_selected != select_peri {
            GpioPin::SerialSelectPeriPin
                .write(if select_peri { Level::Low } else { Level::High });
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
        info!("waiting for not busy");
        while GpioPin::BusyPin.read() == Level::Low {
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
        for l in [Level::High, Level::Low, Level::High, Level::Low, Level::High] {
            GpioPin::ResetPin.write(l);
            sleep(Duration::from_millis(30));
        }
    }
}

impl EPaperDisplayBcmDriver {
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

impl Drop for EPaperDisplayBcmDriver {
    fn drop(&mut self) {
        // we're going to ignore errors here...
        let _ = GpioPin::SerialSelectMainPin.write(Level::Low);
        let _ = GpioPin::SerialSelectPeriPin.write(Level::Low);
        let _ = GpioPin::DataCommandPin.write(Level::Low);
        let _ = GpioPin::ResetPin.write(Level::Low);
        let _ = GpioPin::PowerPin.write(Level::Low);
    }
}
