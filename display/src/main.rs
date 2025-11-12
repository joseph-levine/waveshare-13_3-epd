mod display_constants;
mod e_paper_display_driver;
mod last_run;

use crate::e_paper_display_driver::bit_bang_driver::{EpdError, EPaperDisplayBBDriver as Driver};
use crate::last_run::{update_last_run, safe_to_run};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, info};
use tracing::metadata::LevelFilter;

#[derive(Debug, Parser)]
struct Args {
    file: Option<PathBuf>,
}

#[derive(Debug, Error)]
enum DisplayError {
    #[error(transparent)]
    EpdError(#[from] EpdError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Display has refreshed too recently.")]
    TooSoon,
}

fn main() -> Result<(), DisplayError> {
    if !safe_to_run() {
        return Err(DisplayError::TooSoon);
    }
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
    } else {
        info!("Clearing display");
        device.clear();
    }
    if let Err(e) = update_last_run() {
        error!("{}", e);
    }
    info!("Screen clear. Waiting 2s...");
    sleep(Duration::from_secs(2));
    info!("Dropping device...");
    drop(device);
    info!("Complete");

    Ok(())
}
