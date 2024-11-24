use actix_multipart::{Field, Multipart};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use chrono::Local;
use clap::Parser;
use futures::StreamExt;
use log::{debug, error, info, warn};
use std::{path, str::FromStr};

use util::schema::{Action, UploadForm};

mod api;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    #[arg(long, default_value_t = 9091)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::parse();
    info!("Start sync-server. Args={:?}", args);

    HttpServer::new(|| App::new().service(api::ping_api).service(api::upload_api))
        .bind((args.host, args.port))
        .expect("failed to bind addr")
        .run()
        .await
}
