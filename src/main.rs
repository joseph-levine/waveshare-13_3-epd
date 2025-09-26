extern crate core;

mod e_paper_display;
mod color;

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
    let epd_image = rgb_to_display_u8(&img);

    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    device.send_image(&epd_image)?;
    device.sleep_display()?;

    Ok(())
}
