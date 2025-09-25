use image::imageops::ColorMap;
use image::Rgb;
use palette::{rgb, Lab};
use palette::color_difference::HyAb;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DisplayColor {
    Black = 0x00,
    White = 0x01,
    Yellow = 0x02,
    Red = 0x03,
    Blue = 0x04,
    Green = 0x05,
}

pub struct EPaperMap {
    colormap: Vec<Lab>,
}

impl EPaperMap {
    pub fn new() -> Self {
        Self {
            colormap: vec![
                Lab::from(rgb::Rgb::from([0, 0, 0])),
                Lab::from(rgb::Rgb::from([255, 255, 255])),
                Lab::from(rgb::Rgb::from([255, 243, 56])),
                Lab::from(rgb::Rgb::from([191, 0, 0])),
                Lab::from(rgb::Rgb::from([100, 64, 255])),
                Lab::from(rgb::Rgb::from([67, 138, 28])),
            ]
        }
    }
}

impl ColorMap for EPaperMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let lab_color = Lab::from(rgb::Rgb::from(color.0));
        let distances = self.colormap.map(|l| l.hybrid_distance(lab_color));
        distances.position(distances.min())
    }

    fn map_color(&self, color: &mut Self::Color) {
        todo!()
    }
}