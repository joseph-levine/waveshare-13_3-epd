mod display_color;
pub mod e_paper_color_map;

use crate::color::display_color::DisplayColor;
use image::RgbImage;

pub fn rgb_to_display_4bit(rgb: &RgbImage) -> Vec<u8> {
    rgb.pixels()
        .map(DisplayColor::from)
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|pixels| (pixels[0] as u8) << 4 | pixels[1] as u8) /* pack the first byte as the upper 4 and the second as the lower 4 */
        .collect()
}
