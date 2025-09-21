mod e_paper_display;
mod constants;

use std::error::Error;
use rppal::gpio::{Gpio};
use rppal::spi::Bus::Spi0;
use rppal::spi::{SlaveSelect as ChipSelect, Spi};
use rppal::spi::Mode::Mode0;
use crate::e_paper_display::EpdController;

fn main() {
    let gpio = Gpio::new();
    let spi_main = Spi::new(Spi0, ChipSelect::Ss0, 32_000_000, Mode0);
    let spi_peri = Spi::new(Spi0, ChipSelect::Ss1, 32_000_000, Mode0);
}
