extern crate core;

mod color;
mod display_constants;

use crate::color::{rgb_to_display_4bit, e_paper_color_map::EPaperColorMap};
use clap::Parser;
use image::imageops::{dither, FilterType};
use image::EncodableLayout;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;
use tracing::level_filters::LevelFilter;
use crate::display_constants::{PIXEL_HEIGHT, PIXEL_WIDTH};

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
    out_file: PathBuf,
    #[clap(long)]
    dithered_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG).init();
    let args = Args::parse();

    let img = image::open(&args.file)?;
    info!("Opened image {}", &args.file.display());
    let img = img.resize_to_fill(PIXEL_WIDTH, PIXEL_HEIGHT, FilterType::Lanczos3);
    info!("Resized");
    let img = img.rotate270();
    info!("Rotated");
    let mut img = img.into_rgb8();
    info!("To rgb 8 bit");

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);
    info!("Dithered");
    if let Some(dither_path) = args.dithered_file {
        img.save(dither_path)?;
        info!("Saved dithered image");
    }

    let epd_image = rgb_to_display_4bit(&img);
    info!("Image packed to 4bit format");

    let mut file = File::create(&args.out_file)?;
    file.write_all(epd_image.as_bytes())?;
    info!("Image written. Done");
    Ok(())
}
