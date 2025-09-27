extern crate core;

mod e_paper_display;
mod color;

use clap::Parser;
use e_paper_display::EpdDevice;
use image::imageops::{dither, FilterType};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use crate::e_paper_display::{HEIGHT, WIDTH};

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let epd_image = fs::read(&args.file)?;

    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    device.send_image(&epd_image)?;
    device.sleep_display()?;

    Ok(())
}
