use crate::display_constants::{
    EPD_BYTES_TOTAL, EPD_BYTE_WIDTH_PER_CHIP, EPD_PIXEL_HEIGHT, EPD_TOTAL_BYTES_PER_CHIP,
};
use crate::e_paper_display_driver::bcm2835::{
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_INPT,
    bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP,
    bcm2835_gpio_fsel,
    bcm2835_gpio_lev,
    //     // bcm2835_spi_setClockDivider, bcm2835_spi_setDataMode, bcm2835_spi_transfer,
    //     // bcm2835_spi_transfernb,
    // bcm2835SPIClockDivider_BCM2835_SPI_CLOCK_DIVIDER_128, bcm2835SPIMode_BCM2835_SPI_MODE0,
    bcm2835_gpio_write,
    bcm2835_init,
};
use crate::e_paper_display_driver::gpio_pin::{GpioReadWrite, Level};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::os::raw::{c_char, c_int};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

static EPD_SCK_PIN: u8 = 11;
static EPD_SIO_PIN: u8 = 10;

static EPD_CS_M_PIN: u8 = 8;
static EPD_CS_S_PIN: u8 = 7;

static EPD_DC_PIN: u8 = 25;
static EPD_RST_PIN: u8 = 17;
static EPD_BUSY_PIN: u8 = 24;
static EPD_PWR_PIN: u8 = 18;
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
    pub fn send_data(&self, data: u8) {
        let mut j: u8 = data;
        unsafe {
            bcm2835_gpio_write(EPD_SIO_PIN, 1);
        }
        for _ in 0..8 {
            unsafe {
                bcm2835_gpio_write(EPD_SCK_PIN, 0);
            }
            if j & 0x80 != 0 {
                unsafe {
                    bcm2835_gpio_write(EPD_SIO_PIN, 1);
                }
            } else {
                unsafe {
                    bcm2835_gpio_write(EPD_SIO_PIN, 0);
                }
            }

            unsafe {
                bcm2835_gpio_write(EPD_SCK_PIN, 1);
            }
            j = j << 1;
        }
        unsafe {
            bcm2835_gpio_write(EPD_SCK_PIN, 0);
        }
    }

    pub fn reset(&self) {
        unsafe {
            bcm2835_gpio_write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(EPD_RST_PIN, 0);
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(EPD_RST_PIN, 0);
            sleep(Duration::from_millis(3));
            bcm2835_gpio_write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
        }
    }
    pub fn cs_all(&self, value: u8) {
        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, value);
            bcm2835_gpio_write(EPD_CS_S_PIN, value);
        }
    }

    pub fn send_data2(&self, data: &[u8]) {
        unsafe {
            for byte in data {
                self.send_data(*byte);
            }
        }
    }
    pub fn read_busy_h(&self) {
        info!("e-Paper busy H");
        unsafe {
            while bcm2835_gpio_lev(EPD_BUSY_PIN) == 0 {
                //:      # 0: busy, 1: idle
                sleep(Duration::from_millis(5));
            }
        }
        info!("e-Paper busy H release");
    }
    pub fn turn_on_display(&self) {
        info!("Write PON");
        self.cs_all(0);
        self.send_data(0x04);
        self.cs_all(1);
        self.read_busy_h();

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.cs_all(0);
        self.send_data(0x12);
        self.send_data(0x00);
        self.cs_all(1);
        self.read_busy_h();

        info!("Write POF");
        self.cs_all(0);
        self.send_data(0x02);
        self.send_data(0x00);
        self.cs_all(1);
        info!("Display Done!!");
    }
    pub fn init(&self) {
        info!("EPD init...");
        self.module_init();

        self.reset();
        self.read_busy_h();

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0x74);
        self.send_data(0xC0);
        self.send_data(0x1C);
        self.send_data(0x1C);
        self.send_data(0xCC);
        self.send_data(0xCC);
        self.send_data(0xCC);
        self.send_data(0x15);
        self.send_data(0x15);
        self.send_data(0x55);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0xF0);
        self.send_data(0x49);
        self.send_data(0x55);
        self.send_data(0x13);
        self.send_data(0x5D);
        self.send_data(0x05);
        self.send_data(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0x00);
        self.send_data(0xDF);
        self.send_data(0x69);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0x50);
        self.send_data(0xF7);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0x60);
        self.send_data(0x03);
        self.send_data(0x03);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0x86);
        self.send_data(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0xE3);
        self.send_data(0x22);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0xE0);
        self.send_data(0x01);
        self.cs_all(1);

        self.cs_all(0);
        self.send_data(0x61);
        self.send_data(0x04);
        self.send_data(0xB0);
        self.send_data(0x03);
        self.send_data(0x20);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0x01);
        self.send_data(0x0F);
        self.send_data(0x00);
        self.send_data(0x28);
        self.send_data(0x2C);
        self.send_data(0x28);
        self.send_data(0x38);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0xB6);
        self.send_data(0x07);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0x06);
        self.send_data(0xE8);
        self.send_data(0x28);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0xB7);
        self.send_data(0x01);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0x05);
        self.send_data(0xE8);
        self.send_data(0x28);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0xB0);
        self.send_data(0x01);
        self.cs_all(1);

        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0xB1);
        self.send_data(0x02);
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
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
        }
        self.send_data(0x10);
        for (i, chunk) in image.chunks(EPD_BYTE_WIDTH_PER_CHIP).enumerate() {
            if i % 2 == 0 {
                self.send_data2(chunk);
            }
        }
        self.cs_all(1);
        unsafe {
            bcm2835_gpio_write(EPD_CS_S_PIN, 0);
        }
        self.send_data(0x10);
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
        self.send_data(0x07);
        self.send_data(0xa5);
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
            bcm2835_gpio_fsel(EPD_SCK_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_SIO_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_CS_M_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_CS_S_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_DC_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_RST_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);
            bcm2835_gpio_fsel(EPD_BUSY_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_INPT as u8);
            bcm2835_gpio_fsel(EPD_PWR_PIN, bcm2835FunctionSelect_BCM2835_GPIO_FSEL_OUTP as u8);

            bcm2835_gpio_write(EPD_SCK_PIN, 0);
            bcm2835_gpio_write(EPD_SIO_PIN, 0);
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
            bcm2835_gpio_write(EPD_CS_S_PIN, 0);
            bcm2835_gpio_write(EPD_DC_PIN, 0);
            bcm2835_gpio_write(EPD_RST_PIN, 0);
            bcm2835_gpio_write(EPD_PWR_PIN, 1);
        }
    }

    pub fn module_exit(&self) {
        unsafe {
            bcm2835_gpio_write(EPD_CS_M_PIN, 0);
            bcm2835_gpio_write(EPD_CS_S_PIN, 0);

            bcm2835_gpio_write(EPD_DC_PIN, 0);

            bcm2835_gpio_write(EPD_RST_PIN, 0);
            bcm2835_gpio_write(EPD_PWR_PIN, 0);
        }
    }
}
