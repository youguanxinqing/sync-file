use actix_web::{App, HttpServer};
use clap::Parser;
use log::info;


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
