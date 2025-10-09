mod display_constants;
mod e_paper_display_driver;

use clap::Parser;
use e_paper_display_driver::bcm_driver::EPaperDisplayBcmDriver as Driver;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use tracing::info;
use tracing::metadata::LevelFilter;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG).init();
    let args = Args::parse();
    info!("Reading file...");
    let epd_image = fs::read(&args.file)?;

    let sleep_secs = 20;

    info!("File loaded. Init driver.");
    let mut device = Driver {};
    device.init();
    info!("Device init. Clearing display");
    device.clear();
    info!("Cleared. Sending image...");
    device.display(&epd_image);
    info!("Image sent. Sleeping display...");
    device.sleep();
    info!("Display asleep. Waiting {}s", sleep_secs);
    sleep(Duration::from_secs(sleep_secs));
    info!("Clearing screen");
    device.sleep();
    info!("Screen clear. Waiting 2s...");
    sleep(Duration::from_secs(2));
    info!("Dropping device...");
    drop(device);
    info!("Complete");

    Ok(())
}
