use std::path;

use actix_web::{web, Error, HttpResponse, Responder, Result};
use futures::{future::ok, stream::once};
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
pub struct DownloadReq {
    file_path: String,
}

pub async fn download_file(req: web::Json<DownloadReq>) -> Result<impl Responder> {
    if req.file_path.is_empty() {
        return Ok(HttpResponse::BadRequest().body("invalid file path".to_string()));
    }

    let file_path = path::Path::new(&req.file_path);
    if !file_path.exists() {
        return Ok(HttpResponse::BadRequest().body(format!("not found path: {}", req.file_path)));
    }

    let body = once(ok::<_, Error>(web::Bytes::from(
        fs::read(&req.file_path).await?,
    )));

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body))
}
