extern crate core;

mod e_paper_display;
mod color;

use clap::Parser;
use e_paper_display::EpdDevice;
use std::error::Error;
use std::{fs, mem};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let epd_image = fs::read(&args.file)?;

    let mut device = EpdDevice::new()?;
    device.init()?;
    device.turn_display_on()?;
    device.send_image(&epd_image)?;
    device.sleep_display()?;
    sleep(Duration::from_secs(30));
    device.turn_display_on()?;
    device.clear_screen()?;
    sleep(Duration::from_secs(2));
    drop(device);

    Ok(())
}
