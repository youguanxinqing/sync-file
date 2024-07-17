use clap::Parser;
use std::{fs, path};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    addr: String,

    #[arg(long, default_value_t = String::from("safe"))]
    action: String,

    #[arg(long)]
    local_file_path: Option<String>,

    #[arg(long)]
    remote_file_path: Option<String>,

    #[arg(long, default_value_t = false)]
    ping: bool,
}

fn panic_if_not_expect_file(filename: &str) {
    let path_obj = path::Path::new(filename);
    if !path_obj.exists() {
        panic!("not found local file({:?})", &path_obj);
    }

    if !path_obj.is_file() {
        panic!(
            "local_file_path is not a file({})",
            path_obj.to_str().unwrap_or_default()
        );
    }
}

fn validate_for_upload(args: &Args) {
    panic_if_not_expect_file(&args.local_file_path.clone().unwrap_or_default());
}

fn upload_file(args: &Args) {
    let file_strem = fs::read(args.local_file_path.clone().unwrap().as_str()).unwrap();
    let file_part = reqwest::blocking::multipart::Part::bytes(file_strem)
        .file_name("file")
        .mime_str("text/plain")
        .unwrap();
    let multipart_form = reqwest::blocking::multipart::Form::new()
        .text("action", args.action.clone())
        .text("target_file_path", args.remote_file_path.clone().unwrap())
        .part("file", file_part);

    let url = format!("http://{}/", args.addr);
    let client = reqwest::blocking::Client::new();
    match client.post(url).multipart(multipart_form).send() {
        Err(err) => {
            eprintln!("send request err: {}", err);
            std::process::exit(127);
        }
        Ok(resp) => {
            println!("{}", resp.text().unwrap());
            std::process::exit(0);
        }
    }
}

fn ping_server(args: &Args) {
    let url = format!("http://{}/ping", args.addr);
    match reqwest::blocking::get(url) {
        Err(err) => {
            eprintln!("ping server err: {}", err);
            std::process::exit(127);
        }
        Ok(resp) => {
            println!("{}", resp.text().unwrap_or_default());
            std::process::exit(0);
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("args: {:?}", args);

    if args.ping {
        ping_server(&args);
    } else {
        validate_for_upload(&args);
        upload_file(&args);
    }
}
