mod web_structures;

use actix_files::NamedFile;
use actix_identity::{Identity, IdentityMiddleware};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::http::StatusCode;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::{resource, Form, Path, Query, Redirect};
use actix_web::{
    middleware::Logger, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
    Result as ActixResult,
};
use eink_convert::convert;
use image::imageops::Lanczos3;
use image::metadata::Orientation::NoTransforms;
use image::ImageFormat::Jpeg;
use image::{DynamicImage, ImageDecoder, ImageReader};
use log::error;
use std::collections::HashMap;
use std::env::var;
use std::path::PathBuf;
use tokio::fs::remove_file;
use tokio::process::Command;
use tokio::spawn;
use crate::web_structures::*;

#[derive(Debug, thiserror::Error)]
enum ImageConversionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

fn nybble_img_bin_path(day: ValidDay, hour: ValidHour) -> PathBuf {
    let day: u8 = day.into();
    let hour: u8 = hour.into();
    PathBuf::from("./nybble_images").join(format!("{}/{}.bin", day, hour))
}

fn thumb_path(day: ValidDay, hour: ValidHour, device_pixel_ratio: DevicePixelRatio) -> PathBuf {
    let day: u8 = day.into();
    let hour: u8 = hour.into();
    let device_pixel_ratio: u8 = device_pixel_ratio.into();
    PathBuf::from("./thumbs").join(format!("{}/{}@{}x.jpeg", day, hour, device_pixel_ratio))
}

async fn index(user: Option<Identity>) -> impl Responder {
    if user.is_none() {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    } else {
        HttpResponse::Ok().body(include_str!("../static/index.html"))
    }
}

async fn login_html() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/login.html"))
}

async fn login(req: HttpRequest, auth_data: Form<AuthData>) -> ActixResult<impl Responder> {
    if auth_data.password != var("BASIC_AUTH_PASSWORD").expect("Basic auth not set") {
        return Err(ErrorUnauthorized("Not logged in"));
    }
    Identity::login(&req.extensions(), "user1".to_owned())?;
    Ok(Redirect::to("/").using_status_code(StatusCode::FOUND))
}

async fn pico() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/css/pico.classless.min.css"))
}

async fn save_image(
    day: ValidDay,
    hour: ValidHour,
    file: &TempFile,
) -> Result<(), ImageConversionError> {
    let bin_path = nybble_img_bin_path(day, hour);
    let remove_bin = remove_file(&bin_path).await;
    if let Err(remove_bin) = remove_bin {
        error!(
            "Cannot remove image: {} ({:?})",
            bin_path.display(),
            remove_bin
        );
        // continue anyhow
    }
    let thumb_paths = HashMap::from([
        (
            DevicePixelRatio::One,
            thumb_path(day, hour, DevicePixelRatio::One),
        ),
        (
            DevicePixelRatio::Two,
            thumb_path(day, hour, DevicePixelRatio::Two),
        ),
        (
            DevicePixelRatio::Three,
            thumb_path(day, hour, DevicePixelRatio::Three),
        ),
    ]);
    for (_, thumb_path) in &thumb_paths {
        let remove_thumb = remove_file(thumb_path).await;
        if let Err(remove_thumb) = remove_thumb {
            error!(
                "Cannot remove thumbnail: {} ({:?})",
                thumb_path.display(),
                remove_thumb
            );
            // continue anyhow
        }
    }

    let mut decoder = ImageReader::open(&file.file.path())?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    for (device_pixel_ratio, thumb_path) in &thumb_paths {
        let resized = img.resize(
            256 * (*device_pixel_ratio as u32),
            256 * (*device_pixel_ratio as u32),
            Lanczos3,
        );
        if resized.save_with_format(&thumb_path, Jpeg).is_err() {
            error!("Could not save a thumbnail");
        }
    }
    let binary_conversion = convert(&file.file.path(), &bin_path, None);
    if let Err(err) = binary_conversion {
        error!("Failed to convert file to binary: {}", err);
    }
    Ok(())
}

async fn upload(
    path_parts: Path<(ValidDay, ValidHour)>,
    MultipartForm(form): MultipartForm<UploadMultipartForm>,
    _user: Identity,
) -> ActixResult<impl Responder> {
    let (day, hour) = path_parts.into_inner();
    let display_now = form.json.show_now;
    spawn(async move {
        if save_image(day.into(), hour.into(), &form.file)
            .await
            .is_ok()
            && display_now
        {
            let mut display_cmd = Command::new("/usr/local/bin/eink-display");
            display_cmd.args([nybble_img_bin_path(day.into(), hour.into())]);
            if let Err(e) = display_cmd.spawn() {
                error!("Failed to spawn eink display: {}", e);
            }
        }
    });

    Ok(HttpResponse::Ok())
}

async fn show(
    path_parts: Path<(ValidDay, ValidHour)>,
    _user: Identity,
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

async fn thumbs(
    path_parts: Path<(ValidDay, String)>,
    query: Query<DevicePixelRatioQuery>,
    user: Identity,
) -> ActixResult<impl Responder> {
    let (day, image_name) = path_parts.into_inner();

    let hour = image_name
        .split(".")
        .next()
        .ok_or(ErrorBadRequest("Invalid image name"))?;
    let hour: u8 = hour
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid image name"))?;
    let hour: ValidHour = hour
        .try_into()
        .map_err(|_| ErrorBadRequest("Invalid image name"))?;
    Ok(NamedFile::open_async(thumb_path(day.into(), hour.into(), query.d)).await?)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let secret_key = Key::generate();
    HttpServer::new(move || {
        let host_name = hostname::get().ok().and_then(|s| s.into_string().ok());
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .cookie_name("auth".to_owned())
                    .cookie_secure(false)
                    .cookie_domain(host_name)
                    .cookie_path("/".to_owned())
                    .build(),
            )
            .wrap(Logger::default())
            .service(resource("/upload/{day}/{hour}").post(upload))
            .service(resource("/").get(index))
            .service(resource("/css/pico.classless.min.css").get(pico))
            .service(
                resource("/thumbs/{day}/{image_name}/{density}")
                    .wrap(DefaultHeaders::new().add(("Cache-Control", "max-age=60")))
                    .get(thumbs),
            )
            .service(resource("/show/{day}/{hour}").post(show))
            .service(resource("/login").get(login_html))
            .service(resource("/auth/login").post(login))
    })
    .bind(("0.0.0.0", 80))?
    .workers(2)
    .run()
    .await
}
