use actix_web::{get, HttpResponse, Responder};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Health check request")]
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
