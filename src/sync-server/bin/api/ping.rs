use actix_web::{get, HttpResponse, Responder, Result};


#[get("/ping")]
pub async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Ping OK!!!"))
}
