pub const WIDTH: usize = 1_200;
pub const HEIGHT: usize = 1_600;
pub const HALF_WIDTH: usize = WIDTH / 2;

pub const DISPLAY_BYTES_PER_CHIP: usize = HEIGHT * HALF_WIDTH / 2 /* bytes are packed so one byte is two four-bit color */;