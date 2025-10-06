use std::collections::HashMap;
use image::imageops::ColorMap;
use image::Rgb;
use palette::{FromColor, Oklab, Srgb};
use palette::color_difference::HyAb;
use crate::color::display_color::DisplayColor;

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
        let r = color.0[0] as f32 / u8::MAX as f32;
        let g = color.0[1] as f32 / u8::MAX as f32;
        let b = color.0[2] as f32 / u8::MAX as f32;
        let srgb= Srgb::<f32>::from_components((r,g,b));
        let oklab_color: Oklab<f32> = Oklab::from_color(srgb).into();
        self.colormap
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.hybrid_distance(oklab_color.clone()).total_cmp(&b.hybrid_distance(oklab_color.clone()))
            })
            .map(|(index, _)| index)
            .unwrap_or(&DisplayColor::White)
            .clone() as usize
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        let display_color: DisplayColor = DisplayColor::from(index);
        let srgb = self.colormap.get(&display_color).map(|&v| Srgb::from_color(v));
        if let Some(srgb) = srgb {
            let red = (srgb.red * u8::MAX as f32).round_ties_even() as u8;
            let green = (srgb.green * u8::MAX as f32).round_ties_even() as u8;
            let blue = (srgb.blue * u8::MAX as f32).round_ties_even() as u8;
            return Some(Rgb::from([red, green, blue]));
        }
        None
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