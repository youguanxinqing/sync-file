use actix_multipart::{Field, Multipart};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use chrono::Local;
use clap::Parser;
use futures::StreamExt;
use log::{debug, error, info, warn};
use std::{path, str::FromStr};

use util::schema::{Action, UploadForm};

#[get("/ping")]
pub async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Ping OK!!!"))
}
