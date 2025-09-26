use image::imageops::ColorMap;
use image::{Rgb, RgbImage};
use std::collections::HashMap;

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

impl DisplayColor {
    fn rgb_map() -> HashMap<DisplayColor, Rgb<u8>> {
        HashMap::from([
            (DisplayColor::Black, Rgb::from([0, 0, 0])),
            (DisplayColor::White, Rgb::from([255, 255, 255])),
            (DisplayColor::Yellow, Rgb::from([255, 243, 56])),
            (DisplayColor::Red, Rgb::from([191, 0, 0])),
            (DisplayColor::Blue, Rgb::from([100, 64, 255])),
            (DisplayColor::Green, Rgb::from([67, 138, 28])),
        ])
    }
}

impl From<DisplayColor> for Rgb<u8> {
    fn from(value: DisplayColor) -> Self {
        DisplayColor::rgb_map().get(&value).map(|v| v.clone()).expect("Should be impossible")
    }
}

impl From<&Rgb<u8>> for DisplayColor {
    fn from(value: &Rgb<u8>) -> Self {
        DisplayColor::rgb_map().iter().find_map(|(k, v)| if value == v { Some(k.clone()) } else { None }).unwrap_or_else(|| DisplayColor::White)
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
    colormap: HashMap<DisplayColor, Rgb<u8>>,
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

#[inline(always)]
fn to_u64<'a>(color: &Rgb<u8>) -> [u64; 3] {
    let [r, g, b] = color.0;
    [r as u64, g as u64, b as u64]
}

impl ColorMap for EPaperColorMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let [red, green, blue] = to_u64(color);
        self.colormap
            .iter()
            .min_by(|(_, a), (_, b)| {
                let [ar, ag, ab] = to_u64(a);
                let [br, bg, bb] = to_u64(b);
                ((red - ar).pow(2) + (green - ag).pow(2) + (blue - ab).pow(2))
                    .isqrt()
                    .cmp(&((red - br).pow(2) + (green - bg).pow(2) + (blue - bb).pow(2)).isqrt())
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

pub fn rgb_to_display_u8(rgb: &RgbImage) -> Vec<u8> {
    rgb.pixels().map(|p| DisplayColor::from(p) as u8).collect()
}