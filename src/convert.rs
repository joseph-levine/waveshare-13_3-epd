extern crate core;

mod color;

use color::{rgb_to_display_u8, EPaperColorMap};
use clap::Parser;
use std::fs::File;
use image::imageops::{dither, FilterType};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use image::EncodableLayout;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
    out_file: PathBuf,
    #[clap(long)]
    dithered_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let img = image::open(args.file)?;
    let img = img.resize_to_fill(1200u32, 1600u32, FilterType::Lanczos3);
    let img = img.rotate270();
    let mut img = img.into_rgb8();

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);
    if let Some(dither_path) = args.dithered_file {
        img.save(dither_path)?;
    }

    let epd_image = rgb_to_display_u8(&img);

    let mut file = File::create(&args.out_file)?;
    file.write_all(epd_image.as_bytes())?;
    Ok(())
}
