use image::imageops::ColorMap;
use image::Rgb;
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

impl From<DisplayColor> for Rgb<u8> {
    fn from(value: DisplayColor) -> Self {
        match value {
            DisplayColor::Black => Rgb::from([0, 0, 0]),
            DisplayColor::White => Rgb::from([255, 255, 255]),
            DisplayColor::Yellow => Rgb::from([255, 243, 56]),
            DisplayColor::Red => Rgb::from([191, 0, 0]),
            DisplayColor::Blue => Rgb::from([100, 64, 255]),
            DisplayColor::Green => Rgb::from([67, 138, 28]),
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

impl ColorMap for EPaperColorMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let [red, green, blue] = color.0;
        self.colormap
            .iter()
            .min_by(|(_, a), (_, b)| {
                let [ar, ag, ab] = a.0;
                let [br, bg, bb] = b.0;

                ((red - ar) ^ 2 + (green - ag) ^ 2 + (blue - ab) ^ 2)
                    .isqrt()
                    .cmp(&((red - br) ^ 2 + (green - bg) ^ 2 + (blue - bb) ^ 2).isqrt())
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
