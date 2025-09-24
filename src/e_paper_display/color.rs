use crate::image::color::RGB;

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

impl From<DisplayColor> for RGB<u8> {
    fn from(value: DisplayColor) -> Self {
        match value {
            DisplayColor::Black => RGB { red: 0, green: 0, blue: 0 },
            DisplayColor::White => RGB { red: 255, green: 255, blue: 255 },
            DisplayColor::Yellow => RGB { red: 255, green: 243, blue: 56 },
            DisplayColor::Red => RGB { red: 191, green: 0, blue: 0 },
            DisplayColor::Blue => RGB { red: 100, green: 64, blue: 255 },
            DisplayColor::Green => RGB { red: 67, green: 138, blue: 28 },
        }
    }
}