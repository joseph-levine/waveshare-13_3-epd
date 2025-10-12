extern crate core;

mod color;
mod display_constants;

use crate::color::{e_paper_color_map::EPaperColorMap, rgb_to_display_4bit};
use crate::display_constants::{PIXEL_HEIGHT, PIXEL_WIDTH};
use image::imageops::{dither, FilterType};
use image::{EncodableLayout, ImageError};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;
use tracing::level_filters::LevelFilter;

pub fn convert(
    file: PathBuf,
    out_file: PathBuf,
    dithered_file: Option<PathBuf>,
) -> Result<(), ImageError> {
    let img = image::open(&file)?;
    info!("Opened image {}", &file.display());
    let img = img.rotate270();
    info!("Rotated");
    let img = img.resize_to_fill(PIXEL_WIDTH, PIXEL_HEIGHT, FilterType::Lanczos3);
    info!("Resized");
    let mut img = img.into_rgb8();
    info!("To rgb 8 bit");

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);
    info!("Dithered");
    if let Some(dither_path) = dithered_file {
        img.save(dither_path)?;
        info!("Saved dithered image");
    }

    let epd_image = rgb_to_display_4bit(&img);
    info!("Image packed to 4bit format");

    let mut file = File::create(&out_file)?;
    file.write_all(epd_image.as_bytes())?;
    info!("Image written. Done");
    Ok(())
}
