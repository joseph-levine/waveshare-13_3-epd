extern crate core;

mod color;
mod display_constants;
mod e_paper_display_driver;

use clap::Parser;
use e_paper_display_driver::bit_bang_driver::EPaperDisplayBBDriver as Driver;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let epd_image = fs::read(&args.file)?;

    let mut device = Driver::new()?;
    device.boot_display()?;
    info!("booted");
    device.turn_display_on()?;
    info!("turned on");
    device.send_image(&epd_image)?;
    info!("image sent");
    device.sleep_display()?;
    info!("sleeping for 30s");
    sleep(Duration::from_secs(30));
    device.turn_display_on()?;
    info!("display back on");
    device.clear_screen()?;
    info!("display to white");
    sleep(Duration::from_secs(2));
    drop(device);
    info!("done");

    Ok(())
}
