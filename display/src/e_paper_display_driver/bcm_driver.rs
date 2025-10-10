use crate::display_constants::{EPD_BYTES_TOTAL, EPD_BYTE_WIDTH_PER_CHIP, EPD_PIXEL_HEIGHT, EPD_TOTAL_BYTES_PER_CHIP};
// use crate::e_paper_display_driver::bcm2835::{
//     // bcm2835SPIClockDivider_BCM2835_SPI_CLOCK_DIVIDER_128, bcm2835SPIMode_BCM2835_SPI_MODE0,
//     bcm2835_gpio_fsel,
//     bcm2835_gpio_lev,
//     bcm2835_gpio_write,
//     bcm2835_init,
//     // bcm2835_spi_setClockDivider, bcm2835_spi_setDataMode, bcm2835_spi_transfer,
//     // bcm2835_spi_transfernb,
// };
use crate::e_paper_display_driver::gpio_pin::{GpioReadWrite, Level};
use crate::e_paper_display_driver::waveshare::{
    DEV_Digital_Read, DEV_Digital_Write, DEV_ModuleExit, DEV_ModuleInit, DEV_SPI_SendData,
    DEV_SPI_SendData_nByte,
};
use crate::e_paper_display_driver::{command_code::CommandCode, gpio_pin::GpioPin};
use std::cmp::PartialEq;
use std::io::Error as IoError;
use std::os::raw::{c_char, c_int};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

static EPD_SCK_PIN: u16 = 11;
static EPD_MOSI_PIN: u16 = 10;

static EPD_CS_M_PIN: u16 = 8;
static EPD_CS_S_PIN: u16 = 7;

static EPD_DC_PIN: u16 = 25;
static EPD_RST_PIN: u16 = 17;
static EPD_BUSY_PIN: u16 = 24;
static EPD_PWR_PIN: u16 = 18;
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
    pub fn reset(&self) {
        unsafe {
            DEV_Digital_Write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
            DEV_Digital_Write(EPD_RST_PIN, 0);
            sleep(Duration::from_millis(3));
            DEV_Digital_Write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
            DEV_Digital_Write(EPD_RST_PIN, 0);
            sleep(Duration::from_millis(3));
            DEV_Digital_Write(EPD_RST_PIN, 1);
            sleep(Duration::from_millis(3));
        }
    }
    pub fn cs_all(&self, value: u8) {
        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, value);
            DEV_Digital_Write(EPD_CS_S_PIN, value);
        }
    }
    pub fn send_command(&self, command: u8) {
        unsafe {
            DEV_SPI_SendData(command);
        }
    }
    pub fn send_data(&self, data: u8) {
        unsafe {
            DEV_SPI_SendData(data);
        }
    }
    pub fn send_data2(&self, data: &[u8]) {
        let c_data = data.as_ptr();
        unsafe {
            DEV_SPI_SendData_nByte(c_data, data.len() as u32);
        }
    }
    pub fn read_busy_h(&self) {
        info!("e-Paper busy H");
        unsafe {
            while DEV_Digital_Read(EPD_BUSY_PIN) == 0 { //:      # 0: busy, 1: idle
                sleep(Duration::from_millis(5));
            }
        }
        info!("e-Paper busy H release");
    }
    pub fn turn_on_display(&self) {
        info!("Write PON");
        self.cs_all(0);
        self.send_command(0x04);
        self.cs_all(1);
        self.read_busy_h();

        sleep(Duration::from_millis(50));

        info!("Write DRF");
        self.cs_all(0);
        self.send_command(0x12);
        self.send_data(0x00);
        self.cs_all(1);
        self.read_busy_h();

        info!("Write POF");
        self.cs_all(0);
        self.send_command(0x02);
        self.send_data(0x00);
        self.cs_all(1);
        info!("Display Done!!");
    }
    pub fn init(&self) {
        info!("EPD init...");
        unsafe {
            DEV_ModuleInit();
        }

        self.reset();
        self.read_busy_h();

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0x74);
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
        self.send_command(0xF0);
        self.send_data(0x49);
        self.send_data(0x55);
        self.send_data(0x13);
        self.send_data(0x5D);
        self.send_data(0x05);
        self.send_data(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0x00);
        self.send_data(0xDF);
        self.send_data(0x69);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0x50);
        self.send_data(0xF7);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0x60);
        self.send_data(0x03);
        self.send_data(0x03);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0x86);
        self.send_data(0x10);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0xE3);
        self.send_data(0x22);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0xE0);
        self.send_data(0x01);
        self.cs_all(1);

        self.cs_all(0);
        self.send_command(0x61);
        self.send_data(0x04);
        self.send_data(0xB0);
        self.send_data(0x03);
        self.send_data(0x20);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0x01);
        self.send_data(0x0F);
        self.send_data(0x00);
        self.send_data(0x28);
        self.send_data(0x2C);
        self.send_data(0x28);
        self.send_data(0x38);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0xB6);
        self.send_data(0x07);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0x06);
        self.send_data(0xE8);
        self.send_data(0x28);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0xB7);
        self.send_data(0x01);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0x05);
        self.send_data(0xE8);
        self.send_data(0x28);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0xB0);
        self.send_data(0x01);
        self.cs_all(1);

        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0xB1);
        self.send_data(0x02);
        self.cs_all(1);
    }
    pub fn clear(&self) {
        let zeros: &[u8; EPD_BYTES_TOTAL] = &[0u8; EPD_BYTES_TOTAL];
        self.display(zeros);
        sleep(Duration::from_millis(500));
    }
    pub fn display(&self, image: &[u8]) {

        assert_eq!(image.len(), EPD_BYTES_TOTAL);
        unsafe {
            DEV_Digital_Write(EPD_CS_M_PIN, 0);
        }
        self.send_command(0x10);
        for (i, chunk) in image.chunks(EPD_BYTE_WIDTH_PER_CHIP).enumerate() {
            if i % 2 == 0 {
                self.send_data2(chunk);
            }
        }
        self.cs_all(1);
        unsafe {
            DEV_Digital_Write(EPD_CS_S_PIN, 0);
        }
        self.send_command(0x10);
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
        self.send_command(0x07);
        self.send_data(0xa5);
        self.cs_all(1);

        sleep(Duration::from_millis(2000));
        unsafe {
            DEV_ModuleExit();
        }
    }
}
