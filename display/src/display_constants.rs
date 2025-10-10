/// bytes EPD_are packed so one byte is two four-bit colors. so 600 bytes for 1200 px
pub const EPD_BYTE_WIDTH: usize = 600 ;
pub const EPD_PIXEL_HEIGHT: usize = 1_600; // no packing here, so no second variable
pub const EPD_BYTE_WIDTH_PER_CHIP: usize = EPD_BYTE_WIDTH / 2;
pub const EPD_BYTES_TOTAL: usize = EPD_PIXEL_HEIGHT * EPD_BYTE_WIDTH;
pub const EPD_TOTAL_BYTES_PER_CHIP: usize = EPD_PIXEL_HEIGHT * EPD_BYTE_WIDTH_PER_CHIP ;

