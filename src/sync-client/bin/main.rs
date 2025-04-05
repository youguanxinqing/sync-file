use clap::Parser;
use std::{collections::HashMap, fs, path};

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

    #[arg(long, help = "Specify HOST in request header")]
    host: Option<String>,

    #[arg(long)]
    file_mappings: Option<String>,

    #[arg(long, default_value_t = false)]
    ping: bool,

    #[arg(
        long,
        default_value_t = false,
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

    let url = format!("{}://{}/upload", cfg.protocol.data(), args.addr);
    let client = cfg.protocol.new_client()?;
    let request = (if args.host.is_some() {
        client.post(url).header("Host", args.host.clone().unwrap())
    } else {
        client.post(url)
    })
    .multipart(multipart_form);
    match request.send() {
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

// mappings format: local_file1:remote_file1,local_file2:remote_file2,...
// return {
//   local_file1: remote_file1,
//   local_file2: remote_file2,
// }
fn parse_file_mappings(mappings: &str) -> anyhow::Result<HashMap<String, String>> {
    let mut m: HashMap<String, String> = HashMap::new();
    mappings.split(",").for_each(|chunk| {
        let kv_list: Vec<&str> = chunk.split(":").collect();
        let (local_file, remote_file) = (kv_list[0], kv_list[1]);
        m.insert(local_file.into(), remote_file.into());
    });
    Ok(m)
}

fn upload_file_mappings(args: &Args, cfg: &Config) -> anyhow::Result<()> {
    // parse file mappings
    let mappings = parse_file_mappings(args.file_mappings.as_ref().unwrap())?;

    // http client
    let url = format!("{}://{}/upload", cfg.protocol.data(), args.addr);
    let client = cfg.protocol.new_client()?;

    let mut fail_list = Vec::new();

    for (local_file, remote_file) in mappings.into_iter() {
        let file_strem = fs::read(&local_file).unwrap();
        let file_part = reqwest::blocking::multipart::Part::bytes(file_strem)
            .file_name("file")
            .mime_str("text/plain")
            .unwrap();
        let multipart_form = reqwest::blocking::multipart::Form::new()
            .text("action", args.action.clone())
            .text("target_file_path", remote_file)
            .part("file", file_part);

        // make http request
        let request = (if args.host.is_some() {
            client.post(&url).header("Host", args.host.clone().unwrap())
        } else {
            client.post(&url)
        })
        .multipart(multipart_form);
        match request.send() {
            Err(err) => {
                fail_list.push(format!("{}:{}", local_file, err));
            }
            Ok(resp) => {
                println!("{}: {}", local_file, resp.text().unwrap());
            }
        }
    }

    if !fail_list.is_empty() {
        anyhow::bail!("{}", fail_list.join("\n"));
    }

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
    Http(String),
    Https(String),
}

impl ReqProtocol {
    fn new(protocol: impl Into<String>) -> Self {
        let protocol: String = protocol.into();
        let protocol: &str = &protocol.to_lowercase();
        match protocol {
            "http" => Self::Http(protocol.to_string()),
            "https" => Self::Https(protocol.to_string()),
            _ => unreachable!(),
        }
    }

    fn data(&self) -> String {
        match self {
            Self::Http(data) => data.clone(),
            Self::Https(data) => data.clone(),
        }
    }

    fn new_client(&self) -> anyhow::Result<reqwest::blocking::Client> {
        match self {
            ReqProtocol::Http(_) => reqwest::blocking::Client::builder()
                .build()
                .map_err(anyhow::Error::from),
            ReqProtocol::Https(_) => reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(anyhow::Error::from),
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
    } else if args.file_mappings.is_some() {
        upload_file_mappings(&args, &cfg)?;
    } else {
        validate_for_upload(&args);
        upload_file(&args, &cfg)?;
    }

    Ok(())
}
