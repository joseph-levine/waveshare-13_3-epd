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
use tracing::metadata::LevelFilter;

#[derive(Debug, Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();
    let args = Args::parse();
    info!("Reading file...");
    info!("File loaded. Init driver.");
    let mut device = Driver::new()?;

    info!("Device init");
    if let Some(file) = args.file {
        let epd_image = fs::read(file)?;

        info!("Cleared. Sending image...");
        device.display(&epd_image);
        info!("Image sent. Sleeping display...");
        device.sleep();
    }
    else {
        info!("Clearing display");
        device.clear();
    }
    info!("Screen clear. Waiting 2s...");
    sleep(Duration::from_secs(2));
    info!("Dropping device...");
    drop(device);
    info!("Complete");

    Ok(())
}
