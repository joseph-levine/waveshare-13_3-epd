use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use eink_convert::convert;

#[derive(Parser)]
struct Args {
    file_input: PathBuf,
    file_output: PathBuf,
    dithered_output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    convert(&args.file_input, &args.file_output, args.dithered_output.as_deref())?;
    Ok(())
}
