use actix_web::{get, HttpResponse, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    log::info!("health_check successful");
    HttpResponse::Ok()
}
