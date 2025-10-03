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

    info!("File loaded. Init driver.");
    let mut device = Driver::new()?;
    info!("Booting display");
    device.boot_display();
    info!("booted");
    device.turn_display_on();
    info!("turned on");
    device.send_image(&epd_image);
    info!("image sent");
    device.sleep_display();
    info!("sleeping for 10s");
    sleep(Duration::from_secs(10));
    device.turn_display_on();
    info!("display back on");
    device.clear_screen();
    info!("display to white");
    sleep(Duration::from_secs(2));
    drop(device);
    info!("done");

    Ok(())
}
