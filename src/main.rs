extern crate core;

mod e_paper_display;

use crate::e_paper_display::color::EPaperColorMap;
use clap::Parser;
use e_paper_display::EpdDevice;
use image::imageops::{dither, FilterType};
use std::error::Error;
use std::path::PathBuf;
use crate::e_paper_display::{HEIGHT, WIDTH};

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let img = image::open(args.file)?;
    let img = img.resize_to_fill(WIDTH as u32, HEIGHT as u32, FilterType::Lanczos3);
    let img = img.rotate270();
    let mut img = img.into_rgb8();

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);

    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    // display
    device.sleep_display()?;

    Ok(())
}
