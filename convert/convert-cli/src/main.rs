use clap::Parser;
use eink_convert::convert;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    file_input: PathBuf,
    file_output: PathBuf,
    dithered_output: Option<PathBuf>,
    #[arg(long)]
    saturate: bool,
    #[arg(long)]
    eq_hist: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    convert(
        &args.file_input,
        &args.file_output,
        args.dithered_output.as_deref(),
        args.eq_hist,
        args.saturate,
    )?;
    Ok(())
}
