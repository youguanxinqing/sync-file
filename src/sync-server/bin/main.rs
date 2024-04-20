use actix_multipart::{Field, Multipart};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use chrono::Local;
use futures::StreamExt;
use log::{debug, info, warn};
use std::{path, str::FromStr};
use clap::Parser;

#[get("/ping")]
async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Ping OK!!!"))
}

#[derive(Debug, Default)]
struct UploadForm {
    action: Action,
    content: String,
    target_file_path: String,
}

#[derive(Debug)]
enum Action {
    Safe,
    Force,
}

impl Action {
    fn from_str(action: &str) -> Self {
        match action {
            "safe" => Action::Safe,
            "fore" => Action::Force,
            _ => Action::Safe,
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Safe
    }
}

#[post("/")]
async fn upload(req: HttpRequest, bytes: web::Payload) -> Result<impl Responder> {
    let mut multipart = Multipart::new(req.headers(), bytes);

    // parse multipart
    let mut form = UploadForm::default();
    // let mut upload_file = fs::File::create(format!("1.txt")).unwrap();
    while let Some(chunk) = multipart.next().await {
        let mut chunk = chunk?;

        let content_disposition = chunk.content_disposition().clone();
        let key = content_disposition.get_name().unwrap_or("");
        let value = read_content_disposition(&mut chunk).await;
        match key {
            "action" => {
                form.action = Action::from_str(&value);
            }
            "file" => {
                form.content = value;
            }
            "target_file_path" => {
                form.target_file_path = value;
            }
            _ => {
                warn!("unknown action '{}'", key);
            }
        }
    }

    if let Err(err) = validate_upload_args(&form) {
        return Ok(HttpResponse::BadRequest().body(format!("validate form err: {}", err)));
    }

    if let Err(err) = match form.action {
        Action::Safe => safe_write(&form).await,
        Action::Force => force_write(&form).await,
    } {
        return Ok(HttpResponse::BadRequest().body(format!("write file err: {}", err)));
    }

    Ok(HttpResponse::Ok().body(format!("Upload Successfully!")))
}

async fn safe_create_backup_dir(dir_path: &str) -> std::result::Result<(), String> {
    let path_obj = path::Path::new(dir_path);
    if path_obj.exists() && !path_obj.is_dir() {
        return Err(format!("backup dir is conflicted: {}", dir_path));
    }

    if !path_obj.exists() {
        if let Err(err) = tokio::fs::create_dir_all(dir_path).await {
            return Err(format!("create backup dir err: {}", err));
        }
    }

    Ok(())
}

async fn safe_write(form: &UploadForm) -> std::result::Result<String, String> {
    let target_path = path::Path::new(&form.target_file_path);
    // Check md5. Return directly if md5 does not change.
    if target_path.exists() {
        let old_content = tokio::fs::read_to_string(&target_path)
            .await
            .unwrap_or_default();
        if if_content_md5_equal(&form.content, &old_content) {
            debug!(
                "file({:?}) is not changed.",
                target_path.file_name().unwrap_or_default()
            );
            return Ok(format!("file is not changed."));
        }
    }

    let filename = target_path.file_name().unwrap().to_str().unwrap();
    let filename_without_ext = path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap_or(filename);
    let dir = target_path.parent().unwrap().to_str().unwrap().to_string();

    // create backup dir
    let dot_backup_dir = format!("{}/.{}", dir, filename_without_ext);
    debug!("if need safe create backup dir: {}", &dot_backup_dir);
    safe_create_backup_dir(&dot_backup_dir).await?;

    // backup file
    if target_path.exists() {
        let backup_file = format!(
            "{}/{}.{}",
            dot_backup_dir,
            filename,
            Local::now().format("%Y%m%d_%H%M%S")
        );
        if let Err(err) = tokio::fs::copy(target_path, &backup_file).await {
            return Err(format!("copy backup err: {}", err));
        }
        debug!("backup file({}) ok", backup_file);
    }

    // truncate && write new content
    if let Err(err) = tokio::fs::write(target_path, &form.content).await {
        return Err(format!("write file err: {}", err));
    }
    debug!("write new content ok, file={:?}", target_path);

    Ok("safe write ok".to_string())
}

fn if_content_md5_equal(new_content: &str, old_content: &str) -> bool {
    let new_md5 = juxt_md5::Md5::from_str(new_content).unwrap();
    let old_md5 = juxt_md5::Md5::from_str(old_content).unwrap();
    debug!("new_md5 is: {}, old_md5 is: {}", new_md5, old_md5);

    new_content == old_content
}

async fn force_write(form: &UploadForm) -> std::result::Result<String, String> {
    let target_path = path::Path::new(&form.target_file_path);
    if let Err(err) = tokio::fs::write(target_path, &form.content).await {
        return Err(format!("write file err: {}", err));
    }
    Ok("force write ok".to_string())
}

fn validate_upload_args(form: &UploadForm) -> std::result::Result<(), String> {
    debug!("{:?}", form);

    if form.target_file_path.is_empty() {
        return Err("target_file_path is empty".to_string());
    }
    if form.target_file_path.split("/").collect::<String>().len() < 2 {
        return Err("require level of dir is more than 2".to_string());
    }

    Ok(())
}

async fn read_content_disposition(chunk: &mut Field) -> String {
    let mut s = String::default();
    while let Some(chunk_content) = chunk.next().await {
        s.push_str(
            String::from_utf8(chunk_content.unwrap_or_default().to_vec())
                .unwrap_or_default()
                .as_ref(),
        );
    }
    return s;
}

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

    HttpServer::new(|| App::new().service(ping).service(upload))
        .bind((args.host, args.port))?
        .run()
        .await
}
