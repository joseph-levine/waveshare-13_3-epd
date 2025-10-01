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
use crate::display_constants::{HEIGHT, WIDTH};

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
    let img = img.resize_to_fill(WIDTH as u32, HEIGHT as u32, FilterType::Lanczos3);
    let img = img.rotate270();
    let mut img = img.into_rgb8();

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);
    if let Some(dither_path) = args.dithered_file {
        img.save(dither_path)?;
    }

    let epd_image = rgb_to_display_4bit(&img);

    let mut file = File::create(&args.out_file)?;
    file.write_all(epd_image.as_bytes())?;
    Ok(())
}
