mod display_color;
pub mod e_paper_color_map;
pub mod color_histogram_eq;

use crate::color::display_color::DisplayColor;
use image::RgbImage;

pub fn rgb_to_display_nybbles(rgb: &RgbImage) -> Vec<u8> {
    let mut pix = Vec::with_capacity(rgb.len() / 2);
    for chunk in
    rgb.pixels()
        .map(DisplayColor::from)
        .collect::<Vec<_>>()
        .chunks(2)
    {
        let packed_byte = (u8::from(chunk[0])) << 4 | u8::from(chunk[1]);
        pix.push(packed_byte);
    }
    pix
}
