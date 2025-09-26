use image::imageops::ColorMap;
use palette::color_difference::{HyAb};
use palette::Oklab;
use std::collections::HashMap;
use image::Pixel;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DisplayColor {
    Black = 0x00,
    White = 0x01,
    Yellow = 0x02,
    Red = 0x03,
    Blue = 0x04,
    Green = 0x05,
}

impl From<DisplayColor> for Oklab {
    fn from(value: DisplayColor) -> Self {
        match value {
            DisplayColor::Black => Oklab::from_components((0., 0., 0.)),
            DisplayColor::White => Oklab::from_components((255., 255., 255.)),
            DisplayColor::Yellow => Oklab::from_components((255., 243., 56.)),
            DisplayColor::Red => Oklab::from_components((191., 0., 0.)),
            DisplayColor::Blue => Oklab::from_components((100., 64., 255.)),
            DisplayColor::Green => Oklab::from_components((67., 138., 28.)),
        }
    }
}

impl From<usize> for DisplayColor {
    fn from(value: usize) -> Self {
        match value {
            0 => DisplayColor::Black,
            1 => DisplayColor::White,
            2 => DisplayColor::Yellow,
            3 => DisplayColor::Red,
            4 => DisplayColor::Blue,
            5 => DisplayColor::Green,
            _ => DisplayColor::White,
        }
    }
}

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

    pub fn default_color(&self) -> &Oklab {
        self.colormap.get(&DisplayColor::White).expect("Infallible")
    }
}

impl ColorMap for EPaperColorMap {
    type Color = Oklab;

    fn index_of(&self, color: &Self::Color) -> usize {
        self.colormap
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.hybrid_distance(color.clone())
                    .total_cmp(&b.hybrid_distance(color.clone()))
            })
            .map(|(index, _)| index)
            .unwrap_or(&DisplayColor::White)
            .clone() as usize
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        let display_color: DisplayColor = DisplayColor::from(index);
        self.colormap.get(&display_color).cloned()
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
