extern crate core;

mod color;
mod display_constants;

use crate::color::{e_paper_color_map::EPaperColorMap, rgb_to_display_4bit};
use crate::display_constants::{PIXEL_HEIGHT, PIXEL_WIDTH};
use image::imageops::{dither, FilterType};
use image::{DynamicImage, EncodableLayout, ImageDecoder, ImageError, ImageReader};
use std::fs::File;
use std::io::Write;
use std::path::{Path};
use image::metadata::Orientation::NoTransforms;
use tracing::info;

pub fn convert(
    file: &Path,
    out_file: &Path,
    dithered_file: Option<&Path>,
) -> Result<(), ImageError> {
    let mut decoder = ImageReader::open(&file)?.with_guessed_format()?.into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    let img = img;
    info!("Opened image {}. Rotating...", &file.display());
    let img = img.rotate90();
    info!("Rotated. Resizing...");
    let img = img.resize_to_fill(PIXEL_WIDTH, PIXEL_HEIGHT, FilterType::Lanczos3);
    info!("Resized. To 8-bit RGB...");
    let mut img = img.into_rgb8();

    info!("Cast. Dithering...");

    let epd_map = EPaperColorMap::new();
    dither(&mut img, &epd_map);
    info!("Dithered");

    if let Some(dither_path) = dithered_file {
        img.save(dither_path)?;
        info!("Saved dithered image");
    }
    info!("Packing bytes...");
    let epd_image = rgb_to_display_4bit(&img);
    info!("Image packed to 4bit format. Saving...");

    let mut file = File::create(&out_file)?;
    file.write_all(epd_image.as_bytes())?;
    info!("Image written. Done");
    Ok(())
}
