use log::{error, warn};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use image::{DynamicImage, ImageDecoder, ImageError, ImageReader};
use image::ImageFormat::Jpeg;
use image::imageops::Lanczos3;
use image::metadata::Orientation::NoTransforms;
use zeromq::{PullSocket, Socket, SocketRecv};
use eink_convert::convert;

#[derive(Debug, Deserialize)]
enum Message {
    Resize {
        source: PathBuf,
        destination: PathBuf,
        max_px: u32,
    },
    ConvertToBin {
        source: PathBuf,
        destination: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let mut socket = PullSocket::new();
    socket.bind("127.0.0.1:5567").await?; // Cerritos
    loop {
        if let Ok(message) = socket.recv().await {
            let Ok(msg): Result<String, _> = message.clone().try_into() else {
                warn!("Could not parse message to string: {:?}", message);
                continue;
            };
            let Ok(passed_msg): Result<Message, _> = serde_json::de::from_str(&msg) else {
                warn!("Could not parse message as json: {:?}", msg);
                continue;
            };
            match passed_msg {
                Message::Resize {
                    source,
                    destination,
                    max_px,
                } => {
                    if !source.exists() {
                        warn!("Source image {:?} does not exist", source);
                        continue;
                    }
                    if let Err(e) = resize(source, destination, max_px) {
                        warn!("Error resizing image: {:?}", e);
                    }
                }
                Message::ConvertToBin {
                    source,
                    destination,
                } => {
                    if !source.exists() {
                        warn!("Source image {:?} does not exist", source);
                        continue;
                    }
                    if let Err(e) = convert(&source, &destination, None, false, false) {
                        warn!("Error converting image to bin: {}", e);
                    }
                }
            }
        }
    }
}

fn resize<P: AsRef<Path>>(source: P, dest: P, px: u32) -> Result<(), ImageError>{
    let mut decoder = ImageReader::open(source.as_ref())?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    let resized = img.resize(px, px, Lanczos3);
    if resized.save_with_format(&dest, Jpeg).is_err() {
        error!("Could not save a thumbnail");
    }
    Ok(())
}
