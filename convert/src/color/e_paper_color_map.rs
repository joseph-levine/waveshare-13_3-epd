use crate::color::display_color::{rgb_to_oklab, DisplayColor};
use image::imageops::ColorMap;
use image::Rgb;
use palette::color_difference::HyAb;
use palette::Oklab;
use std::collections::HashMap;

pub struct EPaperColorMap {
    colormap: HashMap<DisplayColor, Oklab>,
}

impl EPaperColorMap {
    pub fn new() -> Self {
        let colors = vec![
            DisplayColor::Black,
            DisplayColor::White,
            DisplayColor::Yellow,
            DisplayColor::Red,
            DisplayColor::Blue,
            DisplayColor::Green,
        ];
        Self {
            colormap: HashMap::from_iter(colors.into_iter().map(|c| (c, c.into()))),
        }
    }
}

impl ColorMap for EPaperColorMap {
    type Color = Rgb<u8>; // dither requires this to be u8

    fn index_of(&self, color: &Self::Color) -> usize {
        let oklab_color: Oklab = rgb_to_oklab(*color);
        let color = self
            .colormap
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.hybrid_distance(oklab_color.clone())
                    .total_cmp(&b.hybrid_distance(oklab_color.clone()))
            })
            .map(|(index, _)| index)
            .expect("Color not found in map");
        *color as usize
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        let display_color: DisplayColor = DisplayColor::from(index);
        Some(display_color.into())
    }

    fn has_lookup(&self) -> bool {
        true
    }

    fn map_color(&self, color: &mut Self::Color) {
        let c = color.clone();
        let new_color = self.lookup(self.index_of(&c)).expect("Infallible");
        *color = new_color;
    }
}
