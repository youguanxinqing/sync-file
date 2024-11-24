use actix_multipart::{Field, Multipart};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder, Result};
use anyhow::anyhow;
use log::{debug, error, warn};
use tokio::{stream, sync::futures};
use std::{path, str::FromStr};
use serde::Deserialize;

#[derive(Deserialize)]
struct DownloadApiReq {
    source_file_path: String,
}

#[post("/v1/download")]
pub async fn download(request: web::Json<DownloadApiReq>) -> Result<impl Responder> {

    if let Err(e) = validate_args(&request) {
        return Ok(HttpResponse::BadRequest().body(format!("bad request err: {}", e)));
    }

    // TODO donwload file & consider a large file
    
    return Ok(HttpResponse::Ok().body(format!("source_file_path = {}", request.source_file_path)))
}

fn validate_args(args: &DownloadApiReq) -> anyhow::Result<()> {
    if args.source_file_path.len() == 0 {
        return Err(anyhow!("require source_file_path is not empty"));
    }

    let path_obj = path::Path::new(&args.source_file_path);
    if !path_obj.exists() || !path_obj.is_file() {
        return Err(anyhow!("require source_file_path is valid, but not found"));
    }

    Ok(())
}
