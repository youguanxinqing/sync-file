use actix_multipart::{Field, Multipart};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use log::{debug, warn};

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
    Fore,
}

impl Action {
    fn from_str(action: &str) -> Self {
        match action {
            "safe" => Action::Safe,
            "fore" => Action::Fore,
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
    let mut forms = UploadForm::default();
    // let mut upload_file = fs::File::create(format!("1.txt")).unwrap();
    while let Some(chunk) = multipart.next().await {
        let mut chunk = chunk?;

        let content_disposition = chunk.content_disposition().clone();
        let key = content_disposition.get_name().unwrap_or("");
        let value = read_content_disposition(&mut chunk).await;
        match key {
            "action" => {
                forms.action = Action::from_str(&value);
            }
            "file" => {
                forms.content = value;
            }
            "target_file_path" => {
                forms.target_file_path = value;
            }
            _ => {
                warn!("unknown action '{}'", key);
            }
        }
    }

    if let Err(err) = validate_upload_args(&forms) {
        return Ok(HttpResponse::BadRequest().body(err.to_string()));
    }

    match forms.action {
        Action::Safe => safe_write(&forms).await,
        Action::Fore => force_write(&forms).await,
    }

    Ok(HttpResponse::Ok().body(format!("Upload Successfully!")))
}

async fn safe_write(forms: &UploadForm) {}

async fn force_write(forms: &UploadForm) {}

fn validate_upload_args(forms: &UploadForm) -> Result<()> {
    debug!("{:?}", forms);
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| App::new().service(ping).service(upload))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
