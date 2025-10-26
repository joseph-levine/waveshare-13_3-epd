use image::Rgb;
use palette::{IntoColor, Oklab, Srgb};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DisplayColor {
    Black = 0x00,
    White = 0x01,
    Yellow = 0x02,
    Red = 0x03,
    Blue = 0x05,
    Green = 0x06,
}

impl DisplayColor {
    pub fn rgb_map() -> HashMap<DisplayColor, Rgb<u8>> {
        HashMap::from([
            (DisplayColor::Black, Rgb::from([0, 0, 0])),
            (DisplayColor::White, Rgb::from([255, 255, 255])),
            (DisplayColor::Yellow, Rgb::from([255, 243, 57])),
            (DisplayColor::Red, Rgb::from([191, 2, 1])),
            (DisplayColor::Blue, Rgb::from([100, 64, 255])),
            (DisplayColor::Green, Rgb::from([68, 138, 28])),
        ])
    }
}

impl From<DisplayColor> for Rgb<u8> {
    fn from(value: DisplayColor) -> Self {
        DisplayColor::rgb_map()
            .get(&value)
            .map(|v| v.clone())
            .expect("Should be impossible")
    }
}

impl From<DisplayColor> for Oklab {
    fn from(value: DisplayColor) -> Self {
        let rgb: Rgb<u8> = value.into();
        rgb_to_oklab(rgb)
    }
}

impl From<&Rgb<u8>> for DisplayColor {
    fn from(value: &Rgb<u8>) -> Self {
        DisplayColor::rgb_map()
            .iter()
            .find_map(|(k, v)| if value == v { Some(k.clone()) } else { None })
            .expect("Mapping color from rgb failed. (Should be impossible)")
    }
}

impl From<usize> for DisplayColor {
    fn from(value: usize) -> Self {
        match value {
            0 => DisplayColor::Black,
            1 => DisplayColor::White,
            2 => DisplayColor::Yellow,
            3 => DisplayColor::Red,
            5 => DisplayColor::Blue,
            6 => DisplayColor::Green,
            _ => DisplayColor::White,
        }
    }
}

impl From<DisplayColor> for u8 {
    fn from(value: DisplayColor) -> Self {
        match value {
            DisplayColor::Black => 0,
            DisplayColor::White => 1,
            DisplayColor::Yellow => 2,
            DisplayColor::Red => 3,
            DisplayColor::Blue => 5,
            DisplayColor::Green => 6,
        }
    }
}

pub fn rgb_to_oklab(rgb: Rgb<u8>) -> Oklab {
    let r = rgb.0[0] as f32 / u8::MAX as f32;
    let g = rgb.0[1] as f32 / u8::MAX as f32;
    let b = rgb.0[2] as f32 / u8::MAX as f32;
    let srgb = Srgb::from((r, g, b));
    srgb.into_color()
}
