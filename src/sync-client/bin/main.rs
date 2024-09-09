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

    #[arg(
        long,
        default_value_t = true,
        help = "Enable https request of insecure."
    )]
    enable_insecure_ssl: bool,
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

fn upload_file(args: &Args, cfg: &Config) -> anyhow::Result<()> {
    let file_strem = fs::read(args.local_file_path.clone().unwrap().as_str()).unwrap();
    let file_part = reqwest::blocking::multipart::Part::bytes(file_strem)
        .file_name("file")
        .mime_str("text/plain")
        .unwrap();
    let multipart_form = reqwest::blocking::multipart::Form::new()
        .text("action", args.action.clone())
        .text("target_file_path", args.remote_file_path.clone().unwrap())
        .part("file", file_part);

    let url = format!("{}://{}/", cfg.protocol.data(), args.addr);
    let client = cfg.protocol.new_client()?;
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

    #[allow(unreachable_code)]
    Ok(())
}

fn ping_server(args: &Args, cfg: &Config) -> anyhow::Result<()> {
    let url = format!("{}://{}/ping", &cfg.protocol.data(), args.addr);
    let client = cfg.protocol.new_client()?;
    match client.get(url).send() {
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

enum ReqProtocol {
    HTTP(String),
    HTTPS(String),
}

impl ReqProtocol {
    fn new(protocol: impl Into<String>) -> Self {
        let protocol: String = protocol.into();
        let protocol: &str = &protocol.to_lowercase();
        match protocol {
            "http" => Self::HTTP(protocol.to_string()),
            "https" => Self::HTTPS(protocol.to_string()),
            _ => unreachable!(),
        }
    }

    fn data(&self) -> String {
        match self {
            Self::HTTP(data) => data.clone(),
            Self::HTTPS(data) => data.clone(),
        }
    }

    fn new_client(&self) -> anyhow::Result<reqwest::blocking::Client> {
        match self {
            ReqProtocol::HTTP(_) => reqwest::blocking::Client::builder()
                .build()
                .map_err(|e| anyhow::Error::from(e)),
            ReqProtocol::HTTPS(_) => reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| anyhow::Error::from(e)),
        }
    }
}

struct Config {
    protocol: ReqProtocol,
}

impl Config {
    fn new(args: &Args) -> Self {
        Config {
            protocol: ReqProtocol::new(if args.enable_insecure_ssl {
                "https"
            } else {
                "http"
            }),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("args: {:?}", args);

    let cfg = Config::new(&args);

    if args.ping {
        ping_server(&args, &cfg)?;
    } else {
        validate_for_upload(&args);
        upload_file(&args, &cfg)?;
    }

    Ok(())
}
