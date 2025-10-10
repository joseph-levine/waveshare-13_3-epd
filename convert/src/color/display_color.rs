use image::Rgb;
use palette::Oklab;
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

    fn oklab_map() -> HashMap<DisplayColor, Oklab> {
        HashMap::from([
            (DisplayColor::Black, Oklab::from_components((0., 0., 0.))),
            (DisplayColor::White, Oklab::from_components((1., 0., 0.))),
            (
                DisplayColor::Yellow,
                Oklab::from_components((0.945, -0.051, 0.181)),
            ),
            (
                DisplayColor::Red,
                Oklab::from_components((0.505, 0.180, 0.101)),
            ),
            (
                DisplayColor::Blue,
                Oklab::from_components((0.543, 0.0537, -0.256)),
            ),
            (
                DisplayColor::Green,
                Oklab::from_components((0.566, -0.115, 0.107)),
            ),
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
        DisplayColor::oklab_map()
            .get(&value)
            .map(|v| v.clone())
            .expect("Should be impossible")
    }
}

impl From<&Rgb<u8>> for DisplayColor {
    fn from(value: &Rgb<u8>) -> Self {
        DisplayColor::rgb_map()
            .iter()
            .find_map(|(k, v)| if value == v { Some(k.clone()) } else { None })
            .unwrap_or_else(|| DisplayColor::White)
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

impl From<DisplayColor> for u8 {
    fn from(value: DisplayColor) -> Self {
        match value {
            DisplayColor::Black => 0,
            DisplayColor::White => 1,
            DisplayColor::Yellow => 2,
            DisplayColor::Red => 3,
            DisplayColor::Blue => 4,
            DisplayColor::Green => 5,
        }
    }
}
