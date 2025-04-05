use actix_web::{HttpResponse, Responder, Result};

pub async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Ping OK!!!"))
}
