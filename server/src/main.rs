use std::env::var;
use std::path::PathBuf;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::{middleware::Logger, App, HttpResponse, HttpServer, Responder, Result as ActixResult, Error as ActixError, post, get};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{Path};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use eink_convert::convert;

fn upload_dir() -> PathBuf {
    PathBuf::from(var("UPLOAD_PATH").unwrap_or("/Users/josephlevine/Developer/server/uploads".to_string()))
}

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

#[post("/upload/{hour}")]
async fn upload(hour: Path<String>, MultipartForm(form): MultipartForm<UploadForm>) -> ActixResult<impl Responder> {
    let hour = hour.into_inner();
    let Ok(hour_valid) = hour.parse::<u8>() else {
        return Ok(HttpResponse::BadRequest());
    };
    if ![5u8, 12u8, 18u8].contains(&hour_valid) {
        return Ok(HttpResponse::BadRequest());
    }
    let Some(file_name) = form.file.file_name.clone() else {
        return Ok(HttpResponse::BadRequest());
    };
    let Some(extension) = file_name.split('.').last() else {
        return Ok(HttpResponse::BadRequest());
    };
    let mut path = upload_dir().clone();
    path.push(format!("{}.{}", hour, extension));
    if form.file.file.persist(&path).is_err() {
        return Ok(HttpResponse::InternalServerError());
    }
    let mut bin_path = upload_dir().clone();
    bin_path.push(format!("{}.bin", hour));
    if convert(&path, &bin_path, None).is_err() {
        return Ok(HttpResponse::InternalServerError());
    }

    Ok(HttpResponse::Ok())
}

async fn static_auth(
    req: ServiceRequest,
    creds: BasicAuth,
) -> ActixResult<ServiceRequest, (ActixError, ServiceRequest)> {
    let basic_auth_env_password = var("BASIC_AUTH_PASSWORD").expect("No auth password in env. Panicking");
    let Some(password) = creds.password() else {
        return Err((ErrorUnauthorized("nope"), req));
    };
    if basic_auth_env_password != password {
        Err((ErrorUnauthorized("nope"), req))
    } else {
        Ok(req)
    }
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(HttpAuthentication::basic(static_auth))
            .wrap(Logger::default())
            .service(upload)
            .service(index)
            .service(pico)
    })
        .bind(("0.0.0.0", 8080))?
        .workers(1)
        .run()
        .await
}
