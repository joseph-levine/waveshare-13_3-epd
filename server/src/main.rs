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
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use log::{debug, error};
use std::env::var;
use std::path::PathBuf;
use tokio::fs::{copy, create_dir};
use tokio::spawn;
use uuid::Uuid;
use zeromq::{PushSocket, Socket, SocketSend};

fn nybble_img_bin_path(day: ValidDay, hour: ValidHour) -> PathBuf {
    let day: u8 = day.into();
    let hour: u8 = hour.into();
    let upload_path = var("UPLOAD_PATH").expect("Checked on app launch");
    PathBuf::from(upload_path).join(format!("nybble_images/{}/{}.bin", day, hour))
}

fn thumb_path(day: ValidDay, hour: ValidHour, device_pixel_ratio: DevicePixelRatio) -> PathBuf {
    let day: u8 = day.into();
    let hour: u8 = hour.into();
    let device_pixel_ratio: u8 = device_pixel_ratio.into();
    let upload_path = var("UPLOAD_PATH").expect("Checked on app launch");
    PathBuf::from(upload_path).join(format!(
        "thumbs/{}/{}@{}x.jpeg",
        day, hour, device_pixel_ratio
    ))
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
    if auth_data.password != var("APP_PASSWORD").expect("App password not set") {
        return Err(ErrorUnauthorized("Not logged in"));
    }
    Identity::login(&req.extensions(), "user1".to_owned())?;
    Ok(Redirect::to("/").using_status_code(StatusCode::FOUND))
}

async fn pico() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/css/pico.classless.min.css"))
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
        let upload_path_var = var("UPLOAD_PATH").expect("Checked on app launch");
        let upload_path = PathBuf::from(upload_path_var);
        let mut save_path = upload_path.join(format!("originals/{}", Uuid::new_v4().to_string()));
        if let Some(suffix) = file_suffix {
            save_path.add_extension(suffix);
        }
        if let Err(e) = copy(form.file.file, &save_path).await {
            error!("Failed to save file {}: {}", save_path.display(), e);
            return;
        };
        let mut push_sock = PushSocket::new();
        if let Err(e) = push_sock.connect("tcp://127.0.0.1:5567").await {
            error!("Failed to connect to socket: {}", e);
            return;
        }
        let mut messages = vec![
            QueueMessage::Resize {
                source: save_path.clone(),
                destination: thumb_path(day, hour, DevicePixelRatio::One),
                max_px: 256,
            },
            QueueMessage::Resize {
                source: save_path.clone(),
                destination: thumb_path(day, hour, DevicePixelRatio::Two),
                max_px: 256 * 2,
            },
            QueueMessage::Resize {
                source: save_path.clone(),
                destination: thumb_path(day, hour, DevicePixelRatio::Three),
                max_px: 256 * 3,
            },
            QueueMessage::ConvertToBin {
                source: save_path,
                destination: nybble_img_bin_path(day, hour),
            },
        ];
        if display_now {
            messages.push(QueueMessage::Display {
                image_path: nybble_img_bin_path(day, hour),
            })
        }
        for message in messages {
            let Ok(json) = serde_json::to_string(&message) else {
                error!("Failed to convert message to json");
                return;
            };
            if let Err(e) = push_sock.send(json.into()).await {
                error!("Failed to send message: {}", e);
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
    let mut push_sock = PushSocket::new();
    if let Err(e) = push_sock.connect("tcp://127.0.0.1:5567").await {
        error!("Failed to connect to socket: {}", e);
        return Ok(HttpResponse::InternalServerError());
    }
    let message = QueueMessage::Display {
        image_path: nybble_img_bin_path(day, hour),
    };
    let Ok(json) = serde_json::to_string(&message) else {
        error!("Failed to convert message to json");
        return Ok(HttpResponse::InternalServerError());
    };
    if let Err(e) = push_sock.send(json.into()).await {
        error!("Failed to send message: {}", e);
        return Ok(HttpResponse::InternalServerError());
    }
    Ok(HttpResponse::Ok())
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
    var("APP_PASSWORD").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "APP_PASSWORD environment variable was not set",
        )
    })?;
    var("UPLOAD_PATH").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "UPLOAD_PATH environment variable was not set",
        )
    })?;
    var("BASE64_COOKIE_SECRET").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "UPLOAD_PATH environment variable was not set",
        )
    })?;
    let decoded = BASE64_STANDARD
        .decode(var("BASE64_COOKIE_SECRET").expect("Checked"))
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Base64 data was invalid")
        })?;
    let secret_key = Key::derive_from(&decoded);
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
    let upload_path = var("UPLOAD_PATH").expect("Checked on app launch");
    let pb = PathBuf::from(upload_path);
    let _ = create_dir(pb.clone().join("thumbs")).await;
    let _ = create_dir(pb.clone().join("nybble_images")).await;
    let _ = create_dir(pb.clone().join("originals")).await;
    for i in 1..8 {
        let _ = create_dir(pb.clone().join(format!("thumbs/{}", i))).await;
        let _ = create_dir(pb.clone().join(format!("nybble_images/{}", i))).await;
    }
}
