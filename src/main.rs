mod e_paper_display;
mod image;

use e_paper_display::EpdDevice;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    // display
    device.sleep_display()?;
    Ok(())
}
