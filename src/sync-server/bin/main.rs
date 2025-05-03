use actix_web::{web, App, HttpServer};
use clap::Parser;
use log::info;

use lib::apis;

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

    HttpServer::new(|| {
        App::new()
            .route("/ping", web::get().to(apis::ping::ping))
            .route("/", web::post().to(apis::upload::upload))
            .route("/upload", web::post().to(apis::upload::upload))
            .route("/download", web::post().to(apis::download::download_file))
    })
    .bind((args.host, args.port))?
    .run()
    .await
}
