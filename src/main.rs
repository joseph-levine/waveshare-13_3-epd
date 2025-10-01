extern crate core;

mod display_constants;
mod e_paper_display_driver;
mod color;

use clap::Parser;
use e_paper_display_driver::bit_bang_driver::EPaperDisplayBBDriver as Driver;
use std::error::Error;
use std::{fs};
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

    let mut device = Driver::new()?;
    device.boot_display()?;
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
