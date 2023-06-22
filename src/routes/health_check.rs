use actix_web::{get, HttpResponse, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!("request_id {} - health_check successful", %request_id);
    let _request_span_guard = request_span.enter();
    HttpResponse::Ok()
}
