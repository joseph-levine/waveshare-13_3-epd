use actix_files::NamedFile;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::{MultipartForm};
use actix_web::error::{
    ErrorBadRequest, ErrorNotFound, ErrorUnauthorized,
};
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
use log::error;
use std::env::var;
use tokio::fs::remove_file;
use std::path::PathBuf;
use tokio::spawn;

#[derive(Debug, thiserror::Error)]
enum ImageConversionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

fn nyble_img_dir() -> PathBuf {
    PathBuf::from("./nyble_img")
}

fn thumbs_dir() -> PathBuf {
    PathBuf::from("./thumbs")
}

const VALID_HOURS: [u8; 3] = [5, 12, 18];

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart()]
    file: TempFile,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

#[get("/css/pico.classless.min.css")]
async fn pico() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/css/pico.classless.min.css"))
}

async fn save_image(hour: String, file: &TempFile) -> Result<(), ImageConversionError> {
    let bin_path = nyble_img_dir().join(format!("{}.bin", hour));
    let remove_bin = remove_file(&bin_path).await;
    if let Err(remove_bin) = remove_bin {
        error!("Cannot remove image: {}\n{:?}", hour, remove_bin);
        // continue anyhow
    }
    let thumb_path = thumbs_dir().join(format!("{}.jpg", hour));
    let remove_thumb = remove_file(&thumb_path).await;
    if let Err(remove_thumb) = remove_thumb {
        error!("Cannot remove thumbnail: {}\n{:?}", hour, remove_thumb);
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

#[post("/upload/{hour}")]
async fn upload(
    hour: Path<String>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> ActixResult<impl Responder> {
    let hour = hour.into_inner();
    let Ok(hour_valid) = hour.parse::<u8>() else {
        return Err(ErrorBadRequest("Hour not a number"));
    };
    if !VALID_HOURS.contains(&hour_valid) {
        return Err(ErrorBadRequest("Hour not in allowed list of hours"));
    }

    spawn(async move {
        let _ = save_image(hour, &form.file).await;
    });

    Ok(HttpResponse::Ok())
}

#[get("/thumbs/{image_name}")]
async fn thumbs(image_name: Path<String>) -> ActixResult<impl Responder> {
    let image_name = image_name.into_inner();
    let valid_names = VALID_HOURS.map(|u| format!("{}.jpeg", u));
    if valid_names.contains(&image_name) {
        return Ok(NamedFile::open_async(thumbs_dir().join(image_name)).await?);
    }
    Err(ErrorNotFound("Thumbnail not found"))
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
    })
    .bind(("0.0.0.0", 8080))?
    .workers(1)
    .run()
    .await
}
