use crate::display_constants::*;
use crate::e_paper_display_driver::bcm2835::{
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_INPT,
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP,
    bcm2835_gpio_fsel,
    bcm2835_gpio_lev,
    // bcm2835_spi_setClockDivider, bcm2835_spi_setDataMode, bcm2835_spi_transfer,
    // bcm2835_spi_transfernb,
    // bcm2835SPIClockDivider_BCM2835_SPI_CLOCK_DIVIDER_128, bcm2835SPIMode_BCM2835_SPI_MODE0,
    bcm2835_gpio_write,
    bcm2835_init,
};
use crate::e_paper_display_driver::gpio_pin::GpioPin::*;
use crate::e_paper_display_driver::gpio_pin::GpioReadWrite;
use crate::e_paper_display_driver::gpio_pin::Level::{High, Low};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::os::raw::{c_char, c_int};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

//
// #[derive(Debug, Error)]
// pub enum EpdError {
//     #[error(transparent)]
//     Io(#[from] IoError),
//     #[error("broadcom init error")]
//     BcmInitError,
// }
//
// #[derive(Debug, Copy, Clone, PartialEq)]
// enum SelectedChip {
//     Main,
//     Peri,
//     Both,
//     Neither,
// }

#[derive(Debug)]
pub struct EPaperDisplayBcmDriver {}

impl EPaperDisplayBcmDriver {
    pub fn send_datum(&self, data: u8) {
        let mut j: u8 = data;
        unsafe {
            bcm2835_gpio_write(SerialDataPin.into(), High.into());
            for _ in 0..8 {
                bcm2835_gpio_write(SerialClockPin.into(), Low.into());
                bcm2835_gpio_write(
                    SerialDataPin.into(),
                    if j & 0x80 != 0 {
                        High.into()
                    } else {
                        Low.into()
                    },
                );
                bcm2835_gpio_write(SerialClockPin.into(), High.into());
                j = j << 1;
            }
            bcm2835_gpio_write(SerialClockPin.into(), Low.into());
        }
    }

    pub fn reset(&self) {
        unsafe {
            bcm2835_gpio_write(ResetPin.into(), High.into());
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(ResetPin.into(), Low.into());
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(ResetPin.into(), High.into());
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(ResetPin.into(), Low.into());
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(ResetPin.into(), High.into());
            sleep(Duration::from_millis(3));
        }
    }
    pub fn cs_all(&self, value: u8) {
        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), value);
            bcm2835_gpio_write(SerialSelectPeriPin.into(), value);
        }
    }

    pub fn send_data2(&self, data: &[u8]) {
        unsafe {
            for byte in data {
                self.send_datum(*byte);
            }
        }
    }
    pub fn read_busy_h(&self) {
        info!("e-Paper busy H");
        unsafe {
            while bcm2835_gpio_lev(BusyPin.into()) == Low.into() {
                //:      # Low.into(): busy, High.into(): idle
                sleep(Duration::from_millis(5));
            }
        }
        info!("e-Paper busy H release");
    }
    pub fn turn_on_display(&self) {
        info!("Write PON");
        self.cs_all(0);
        self.send_datum(0x04);
        self.cs_all(1);
        self.read_busy_h();

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.cs_all(0);
        self.send_datum(0x12);
        self.send_datum(0x00);
        self.cs_all(1);
        self.read_busy_h();

        info!("Write POF");
        self.cs_all(0);
        self.send_datum(0x02);
        self.send_datum(0x00);
        self.cs_all(1);
        info!("Display Done!!");
    }
    pub fn init(&self) {
        info!("EPD init...");
        self.module_init();

        self.reset();
        self.read_busy_h();

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0x74);
        self.send_datum(0xC0);
        self.send_datum(0x1C);
        self.send_datum(0x1C);
        self.send_datum(0xCC);
        self.send_datum(0xCC);
        self.send_datum(0xCC);
        self.send_datum(0x15);
        self.send_datum(0x15);
        self.send_datum(0x55);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0xF0);
        self.send_datum(0x49);
        self.send_datum(0x55);
        self.send_datum(0x13);
        self.send_datum(0x5D);
        self.send_datum(0x05);
        self.send_datum(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0x00);
        self.send_datum(0xDF);
        self.send_datum(0x69);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0x50);
        self.send_datum(0xF7);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0x60);
        self.send_datum(0x03);
        self.send_datum(0x03);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0x86);
        self.send_datum(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0xE3);
        self.send_datum(0x22);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0xE0);
        self.send_datum(0x01);
        self.cs_all(1);

        self.cs_all(0);
        self.send_datum(0x61);
        self.send_datum(0x04);
        self.send_datum(0xB0);
        self.send_datum(0x03);
        self.send_datum(0x20);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0x01);
        self.send_datum(0x0F);
        self.send_datum(0x00);
        self.send_datum(0x28);
        self.send_datum(0x2C);
        self.send_datum(0x28);
        self.send_datum(0x38);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0xB6);
        self.send_datum(0x07);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0x06);
        self.send_datum(0xE8);
        self.send_datum(0x28);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0xB7);
        self.send_datum(0x01);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0x05);
        self.send_datum(0xE8);
        self.send_datum(0x28);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0xB0);
        self.send_datum(0x01);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0xB1);
        self.send_datum(0x02);
        self.cs_all(1);
    }
    pub fn clear(&self) {
        let zeros: &[u8; EPD_BYTES_TOTAL] = &[1u8; EPD_BYTES_TOTAL];
        self.display(zeros);
        sleep(Duration::from_millis(500));
    }
    pub fn display(&self, image: &[u8]) {
        assert_eq!(image.len(), EPD_BYTES_TOTAL);
        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), 0);
        }
        self.send_datum(0x10);
        for (i, chunk) in image.chunks(EPD_BYTE_WIDTH_PER_CHIP).enumerate() {
            if i % 2 == 0 {
                self.send_data2(chunk);
            }
        }
        self.cs_all(1);
        unsafe {
            bcm2835_gpio_write(SerialSelectPeriPin.into(), 0);
        }
        self.send_datum(0x10);
        for (i, chunk) in image.chunks(EPD_BYTE_WIDTH_PER_CHIP).enumerate() {
            if i % 2 == 1 {
                self.send_data2(chunk);
            }
        }
        self.cs_all(1);
        sleep(Duration::from_millis(100));

        self.turn_on_display();
    }
    pub fn sleep(&self) {
        self.cs_all(0);
        self.send_datum(0x07);
        self.send_datum(0xa5);
        self.cs_all(1);

        sleep(Duration::from_millis(2000));
        self.module_exit();
    }

    pub fn module_init(&self) {
        let r: i32;
        unsafe {
            r = bcm2835_init();
        }
        if r == 0 {
            panic!("BCM2835 init failed.");
        }
        unsafe {
            bcm2835_gpio_fsel(
                SerialClockPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                SerialDataPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                SerialSelectMainPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                SerialSelectPeriPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                DataCommandPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                ResetPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );
            bcm2835_gpio_fsel(
                BusyPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_INPT as u8,
            );
            bcm2835_gpio_fsel(
                PowerPin.into(),
                bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8,
            );

            bcm2835_gpio_write(SerialClockPin.into(), Low.into());
            bcm2835_gpio_write(SerialDataPin.into(), Low.into());
            bcm2835_gpio_write(SerialSelectMainPin.into(), Low.into());
            bcm2835_gpio_write(SerialSelectPeriPin.into(), Low.into());
            bcm2835_gpio_write(DataCommandPin.into(), Low.into());
            bcm2835_gpio_write(ResetPin.into(), Low.into());
            bcm2835_gpio_write(PowerPin.into(), High.into());
        }
    }

    pub fn module_exit(&self) {
        unsafe {
            bcm2835_gpio_write(SerialSelectMainPin.into(), Low.into());
            bcm2835_gpio_write(SerialSelectPeriPin.into(), Low.into());

            bcm2835_gpio_write(DataCommandPin.into(), Low.into());

            bcm2835_gpio_write(ResetPin.into(), Low.into());
            bcm2835_gpio_write(PowerPin.into(), Low.into());
        }
    }
}
