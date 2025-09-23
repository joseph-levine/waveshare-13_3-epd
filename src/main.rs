mod constants;
mod e_paper_display;

use crate::e_paper_display::EpdDevice;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let controller = EpdDevice::new()?;
    controller.clear()?;
    controller.display()?;
    controller.deep_sleep()?;

    Ok(())
}
