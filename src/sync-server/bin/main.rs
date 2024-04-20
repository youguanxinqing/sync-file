use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{cookie::time::format_description::well_known::iso8601::Config, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use std::{cmp::PartialOrd, path::PathBuf};


#[get("/ping")]
async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Ping OK!!!"))
}

#[derive(MultipartForm)]
struct UploadRequest {
    file: TempFile,
}

#[post("/")]
async fn upload(form: MultipartForm<UploadRequest>) -> Result<impl Responder> {
    const MAX_FILE_SIZE: u64 = 1024 * 1024 * 10; // 10M
    const MAX_FILE_COUNT: u32 = 1;

    match form.file.size {
        0 => return Ok(HttpResponse::BadRequest().finish()),
        length if length > (MAX_FILE_SIZE as usize) => {
            return Ok(HttpResponse::BadRequest()
                .body(format!("The uploaded file is too large. Maximum size is {} bytes.", MAX_FILE_SIZE)));
        },
        _ => {}
    }

    let temp_file = form.file.file.path();
    let file_name = form.file.file_name.as_ref().map(|m| m.as_ref()).unwrap_or("null");

    let path_file = PathBuf::from("1.txt");

    Ok(HttpResponse::Ok().body("Upload Successfully!".to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(ping))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
