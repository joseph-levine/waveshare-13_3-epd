mod constants;
mod e_paper_display;

use crate::e_paper_display::EpdDevice;
use std::error::Error;
use linux_embedded_hal::SpidevDevice;

fn main() -> Result<(), Box<dyn Error>> {
    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    // display
    device.sleep_display()?;
    Ok(())
}
