mod e_paper_display;

use crate::e_paper_display::color::EPaperColorMap;
use clap::Parser;
use e_paper_display::EpdDevice;
use image::imageops::dither;
use std::error::Error;
use std::path::PathBuf;
use palette::IntoColor;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let img = image::open(args.file)?;
    let mut rgb_img = img.to_rgba32f();

    let epd_map = EPaperColorMap::new();
    dither(&mut rgb_img, &epd_map);

    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    // display
    device.sleep_display()?;

    Ok(())
}
