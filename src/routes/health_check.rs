use actix_web::{get, HttpResponse, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    let request_id = uuid::Uuid::new_v4();
    log::info!("request_id {} - health_check successful", request_id);
    HttpResponse::Ok()
}
