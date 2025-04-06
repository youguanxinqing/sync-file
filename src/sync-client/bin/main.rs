use chrono::Local;
use clap::{arg, command, ArgAction, Args as ClapArgs, Parser, Subcommand};
use lib::apis::urls;
use log::{debug, error, info, LevelFilter};
use std::io::prelude::Write;
use std::{collections::HashMap, fs, path};

#[derive(ClapArgs, Debug, PartialEq)]
struct Global {
    /// Verbose log
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

impl Global {
    fn log_filter_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(about = "")]
    Push,

    #[command(about = "")]
    Pull(PullArgs),

    #[command(about = "")]
    Test(TestArgs),
}

#[derive(ClapArgs, Debug)]
pub struct TestArgs {
    #[arg(long, default_value_t = false)]
    ping: bool,
}

#[derive(ClapArgs, Debug)]
pub struct PullArgs {
    #[arg(long)]
    file_mappings: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[command(flatten)]
    global: Global,

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

    #[arg(
        long,
        default_value_t = false,
        help = "Enable https request of insecure."
    )]
    enable_insecure_ssl: bool,

    #[command(subcommand)]
    command: SubCommand,
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

    let url = urls::UPLOAD_URL_V1!(cfg.protocol.data(), args.addr);
    let client = cfg.protocol.new_client()?;
    let request = (if args.host.is_some() {
        client.post(url).header("Host", args.host.clone().unwrap())
    } else {
        client.post(url)
    })
    .multipart(multipart_form);
    match request.send() {
        Err(err) => {
            error!("send request err: {}", err);
            std::process::exit(127);
        }
        Ok(resp) => {
            info!("{}", resp.text().unwrap());
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
    let url = urls::UPLOAD_URL_V1!(cfg.protocol.data(), args.addr);
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
                info!("{}: {}", local_file, resp.text().unwrap());
            }
        }
    }

    if !fail_list.is_empty() {
        anyhow::bail!("{}", fail_list.join("\n"));
    }

    Ok(())
}

fn download_file_mappings(args: &PullArgs, cfg: &Config) -> anyhow::Result<()> {
    let mappings = parse_file_mappings(&args.file_mappings)?;
    if mappings.is_empty() {
        anyhow::bail!("--file-mapping is invalid: {}", args.file_mappings);
    };

    let mut fail_list = Vec::new();

    let client = cfg.protocol.new_client()?;
    for (remote_path, local_file) in mappings.iter() {
        // 1. download remote file
        let mut m = HashMap::new();
        m.insert("file_path", remote_path);

        let url = urls::DOWNLOAD_URL_V1!(cfg.protocol.data(), cfg.addr);
        let request = (if cfg.header_host.is_some() {
            client
                .post(url)
                .header("Host", cfg.header_host.clone().unwrap())
        } else {
            client.post(url)
        })
        .json(&m);
        match request.send() {
            Err(err) => {
                fail_list.push(format!("{}:{}", local_file, err));
            }
            Ok(resp) => {
                // 2. write to local file
                fs::write(local_file, resp.bytes()?)?;
            }
        }
    }

    if !fail_list.is_empty() {
        anyhow::bail!("{}", fail_list.join("\n"));
    }

    Ok(())
}

fn ping_server(cfg: &Config) -> anyhow::Result<()> {
    let url = urls::PING_URL_V1!(&cfg.protocol.data(), cfg.addr);
    let client = cfg.protocol.new_client()?;
    match client.get(url).send() {
        Err(err) => {
            error!("ping server err: {}", err);
            std::process::exit(127);
        }
        Ok(resp) => {
            info!(
                "code: {}, {}",
                resp.status(),
                resp.text().unwrap_or_default()
            );
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
    addr: String,
    header_host: Option<String>,
    protocol: ReqProtocol,
}

impl Config {
    fn new(args: &Args) -> Self {
        Config {
            addr: args.addr.clone(),
            header_host: args.host.clone(),
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

    env_logger::builder()
        .format(move |buf, record| {
            let level = record.level();
            let style = buf.default_level_style(level);
            let time = Local::now().format("%Y-%m-%d %H:%M:%S");
            let args = record.args();
            let target = record.target();
            buf.write_fmt(format_args!(
                "[{time} {style}{level}{style:#} {target}] {args}\n",
            ))
        })
        .filter_level(args.global.log_filter_level())
        .init();
    debug!("args: {:?}", args);

    let cfg = Config::new(&args);

    match args.command {
        SubCommand::Test(test_args) => {
            if test_args.ping {
                ping_server(&cfg)?;
            }
        }
        SubCommand::Pull(pull_args) => {
            download_file_mappings(&pull_args, &cfg)?;
        }
        _ => {
            if args.file_mappings.is_some() {
                upload_file_mappings(&args, &cfg)?;
            } else {
                validate_for_upload(&args);
                upload_file(&args, &cfg)?;
            }
        }
    }

    Ok(())
}
