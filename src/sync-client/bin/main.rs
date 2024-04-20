use log::{debug, info, warn};
use clap::Parser;
use std::path;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    target_addr: String,

    #[arg(long, default_value_t = String::from("safe"))]
    action: String,

    #[arg(long)]
    target_file_path: String,

    #[arg(long, default_value_t = false)]
    ping: bool,
}

fn panic_if_no_file(filename: &str) {
    let path_obj = path::Path::new(filename);
    if !path_obj.exists() {
        panic!("Not found file({:?})", &path_obj);
    }
}

fn ping_server() {

}

fn validate_for_upload(args: &Args) {
    panic_if_no_file(&args.target_file_path);
}

fn upload_file(args: &Args) {

}


fn main() {
    env_logger::init();

    let args = Args::parse();
    info!("args: {:?}", args);

    if args.ping {
        ping_server()
    } else {
        validate_for_upload(&args);
        upload_file(&args);
    }
}
