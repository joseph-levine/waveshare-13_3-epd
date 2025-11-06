use actix_files::NamedFile;
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};
use actix_web::error::{ErrorBadRequest, ErrorNotFound, ErrorUnauthorized};
use actix_web::web::Path;
use actix_web::{
    get, middleware::Logger, post, App, HttpResponse, HttpServer, Responder, Result as ActixResult,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use eink_convert::convert;
use image::imageops::Lanczos3;
use image::metadata::Orientation::NoTransforms;
use image::ImageFormat::Jpeg;
use image::{DynamicImage, ImageDecoder, ImageReader};
use log::{error, info};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::env::var;
use std::path::PathBuf;
use tokio::fs::remove_file;
use tokio::process::Command;
use tokio::spawn;

#[derive(Debug, thiserror::Error)]
enum ImageConversionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

fn nybble_img_bin_path(day: u8, hour: u8) -> PathBuf {
    PathBuf::from("./nybble_images").join(format!("{}/{}.bin", day, hour))
}

fn thumb_path(day: u8, hour: u8) -> PathBuf {
    PathBuf::from("./thumbs").join(format!("{}/{}.jpeg", day, hour))
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
enum ValidHour {
    Morning = 5,
    Noon = 12,
    Night = 18
}
impl Into<u8> for ValidHour {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryInto<ValidHour> for u8 {
    type Error = ();

    fn try_into(self) -> Result<ValidHour, Self::Error> {
        match self {
            5 => Ok(ValidHour::Morning),
            12 => Ok(ValidHour::Noon),
            18 => Ok(ValidHour::Night),
            _ => Err(())
        }
    }
}
#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
enum ValidDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Into<u8> for ValidDay {
    fn into(self) -> u8 {
        self as u8
    }
}


#[derive(Debug, Deserialize)]
struct UploadJsonForm {
    show_now: bool
}

#[derive(Debug, MultipartForm)]
struct UploadMultipartForm {
    #[multipart()]
    file: TempFile,
    json: MpJson<UploadJsonForm>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

#[get("/css/pico.classless.min.css")]
async fn pico() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/css/pico.classless.min.css"))
}

async fn save_image(day: u8, hour: u8, file: &TempFile) -> Result<(), ImageConversionError> {
    let bin_path = nybble_img_bin_path(day, hour);
    let remove_bin = remove_file(&bin_path).await;
    if let Err(remove_bin) = remove_bin {
        error!("Cannot remove image: {}/{} ({:?})", day, hour, remove_bin);
        // continue anyhow
    }
    let thumb_path = thumb_path(day, hour);
    let remove_thumb = remove_file(&thumb_path).await;
    if let Err(remove_thumb) = remove_thumb {
        error!(
            "Cannot remove thumbnail: {}/{} ({:?})",
            day, hour, remove_thumb
        );
        // continue anyhow
    }

    let mut decoder = ImageReader::open(&file.file.path())?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    let resized = img.resize(256, 256, Lanczos3);
    if resized.save_with_format(&thumb_path, Jpeg).is_err() {
        error!("Could not save a thumbnail");
    }
    let binary_conversion = convert(&file.file.path(), &bin_path, None);
    if let Err(err) = binary_conversion {
        error!("Failed to convert file to binary: {}", err);
    }
    Ok(())
}

#[post("/upload/{day}/{hour}")]
async fn upload(
    path_parts: Path<(ValidDay, ValidHour)>,
    MultipartForm(form): MultipartForm<UploadMultipartForm>,
) -> ActixResult<impl Responder> {
    let (day, hour) = path_parts.into_inner();
    let display_now = form.json.show_now;
    spawn(async move {
        if save_image(day.into(), hour.into(), &form.file).await.is_ok() && display_now {
            let mut display_cmd = Command::new("/usr/local/bin/eink-display");
            display_cmd.args([nybble_img_bin_path(day.into(), hour.into())]);
            if let Err(e) = display_cmd.spawn() {
                error!("Failed to spawn eink display: {}", e);
            }
        }
    });

    Ok(HttpResponse::Ok())
}

#[post("/show/{day}/{hour}")]
async fn show(
    path_parts: Path<(ValidDay, ValidHour)>
) -> ActixResult<impl Responder> {
    let (day, hour) = path_parts.into_inner();
    spawn(async move {
        let mut display_cmd = Command::new("/usr/local/bin/eink-display");
        display_cmd.args([nybble_img_bin_path(day.into(), hour.into())]);
        if let Err(e) = display_cmd.spawn() {
            error!("Failed to spawn eink display: {}", e);
        }
    });

    Ok(HttpResponse::Ok())
}

#[get("/thumbs/{day}/{image_name}")]
async fn thumbs(path_parts: Path<(ValidDay, String)>) -> ActixResult<impl Responder> {
    let (day, image_name) = path_parts.into_inner();

    let hour = image_name
        .split(".")
        .next()
        .ok_or(ErrorBadRequest("Invalid image name"))?;
    let hour: u8 = hour
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid image name"))?;
    let hour: ValidHour = hour.try_into()
        .map_err(|_| ErrorBadRequest("Invalid image name"))?;
    Ok(NamedFile::open_async(thumb_path(day.into(), hour.into())).await?)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        let auth = HttpAuthentication::basic(|req, credentials| async move {
            if let Some(pass) = credentials.password() {
                if pass == var("BASIC_AUTH_PASSWORD").expect("Basic auth not set") {
                    return Ok(req);
                }
            }
            Err((ErrorUnauthorized("Not Authorized"), req))
        });
        App::new()
            .wrap(Logger::default())
            .wrap(auth)
            .service(upload)
            .service(index)
            .service(pico)
            .service(thumbs)
            .service(show)
    })
    .bind(("0.0.0.0", 80))?
    .workers(2)
    .run()
    .await
}
