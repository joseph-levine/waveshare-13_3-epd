use std::ffi::{c_char, c_int};

unsafe extern "C" {
    fn bcm2835_init() -> c_int;
    fn bcm2835_gpio_write(pin: u8, on: u8);
    fn bcm2835_gpio_lev(pin: u8) -> u8;
    fn bcm2835_spi_transfer(value: u8) -> u8;

    fn bcm2835_spi_transfernb(tbuf: *const c_char, rbuf: *mut c_char, len: u32);
}

pub fn bcm_init() -> i32 {
    let mut result: i32 = 0;
    unsafe {
        result = bcm_init();
    }
    result
}

pub fn bcm_write(pin: u8, on: u8) {
    unsafe {
        bcm2835_gpio_write(pin, on);
    }
}

pub fn bcm_read(pin: u8) -> u8 {
    let mut level: u8 = 0;
    unsafe {
        level = bcm2835_gpio_lev(pin);
    }
    level
}

pub fn bcm_spi_transfer(value: u8) -> u8 {
    let mut result: u8 = 0;
    unsafe {
        result = bcm2835_spi_transfer(value)
    }
    result
}

pub fn bcm_spi_transfer_buf(send: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(send.len());
    let mut result_slice = result.as_slice();
    unsafe { bcm2835_spi_transfernb(send.into(), &mut result_slice, send.len() as u32) };

    result_slice.to_vec()
}
