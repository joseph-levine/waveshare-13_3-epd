mod web_structures;

use crate::web_structures::*;
use actix_files::NamedFile;
use actix_identity::{Identity, IdentityMiddleware};
use actix_multipart::form::MultipartForm;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::cookie::time::Duration;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::http::StatusCode;
use actix_web::middleware::{DefaultHeaders, NormalizePath, TrailingSlash};
use actix_web::web::{Form, Path, Query, Redirect, resource};
use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, Result as ActixResult,
    middleware::Logger,
};
use eink_convert::convert;
use image::ImageFormat::Jpeg;
use image::imageops::Lanczos3;
use image::metadata::Orientation::NoTransforms;
use image::{DynamicImage, ImageDecoder, ImageReader};
use log::{debug, error};
use std::collections::HashMap;
use std::env::var;
use std::path::PathBuf;
use tokio::fs::{create_dir, remove_file};
use tokio::process::Command;
use tokio::spawn;
use uuid::Uuid;

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
    file: &PathBuf,
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

    let mut decoder = ImageReader::open(&file)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    for (device_pixel_ratio, thumb_path) in &thumb_paths {
        let px = (*device_pixel_ratio).into();
        let resized = img.resize(px, px, Lanczos3);
        if resized.save_with_format(&thumb_path, Jpeg).is_err() {
            error!("Could not save a thumbnail");
        }
    }
    let binary_conversion = convert(&file, &bin_path, None, false, false);
    if let Err(err) = binary_conversion {
        error!("Failed to convert file to binary: {}", err);
    }
    Ok(())
}

async fn upload(
    path_parts: Path<(ValidDay, ValidHour)>,
    MultipartForm(form): MultipartForm<UploadMultipartForm>,
    user: Identity,
) -> ActixResult<impl Responder> {
    debug!("Upload for {}", user.id().unwrap_or("none?!".to_owned()));
    let (day, hour) = path_parts.into_inner();
    let display_now = form.json.show_now;
    spawn(async move {
        let file_suffix = form
            .file
            .content_type
            .map(|m| m.suffix().map(|s| s.to_string()))
            .flatten();
        let mut save_path: PathBuf =
            PathBuf::from(format!("./originals/{}", Uuid::new_v4().to_string()));
        if let Some(suffix) = file_suffix {
            save_path.add_extension(suffix);
        }
        match form.file.file.persist(&save_path) {
            Ok(_) => {
                if save_image(day.into(), hour.into(), &save_path)
                    .await
                    .is_ok()
                    && display_now
                {
                    display_e_ink_image(day, hour);
                }
            }
            Err(_) => {
                error!("Could not save file: {}", save_path.display());
            }
        }
    });

    Ok(HttpResponse::Ok())
}

async fn show(
    path_parts: Path<(ValidDay, ValidHour)>,
    user: Identity,
) -> ActixResult<impl Responder> {
    debug!("show for {}", user.id().unwrap_or("none?!".to_owned()));
    let (day, hour) = path_parts.into_inner();
    display_e_ink_image(day, hour);
    Ok(HttpResponse::Ok())
}

fn display_e_ink_image(day: ValidDay, hour: ValidHour) {
    spawn(async move {
        let mut display_cmd = Command::new("/usr/local/bin/eink-display");
        display_cmd.args([nybble_img_bin_path(day.into(), hour.into())]);
        if let Err(e) = display_cmd.spawn() {
            error!("Failed to spawn eink display: {}", e);
        }
    });
}

async fn thumbs(
    path_parts: Path<(ValidDay, String)>,
    query: Query<DevicePixelRatioQuery>,
    user: Identity,
) -> ActixResult<impl Responder> {
    debug!("Thumb for {}", user.id().unwrap_or("none?!".to_owned()));
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
    Ok(NamedFile::open(thumb_path(
        day.into(),
        hour.into(),
        query.d.unwrap_or(DevicePixelRatio::One),
    )))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let secret_key = Key::generate();
    create_app_directories().await;
    HttpServer::new(move || {
        let host_name = hostname::get().ok().and_then(|s| s.into_string().ok());
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
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
                resource("/thumbs/{day}/{image_name}")
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
    // Note to self, use your hostname to connect, not 127 or localhost (for session cookie)
}

async fn create_app_directories() {
    let _ = create_dir("thumbs").await;
    let _ = create_dir("nybble_images").await;
    let _ = create_dir("originals").await;
    for i in 1..8 {
        let _ = create_dir(format!("thumbs/{}", i)).await;
        let _ = create_dir(format!("nybble_images/{}", i)).await;
    }
}
