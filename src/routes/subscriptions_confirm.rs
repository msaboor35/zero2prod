use actix_web::{get, HttpResponse, Responder};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Confirm a pending subscriber")]
#[get("/subscriptions/confirm")]
async fn confirm() -> impl Responder {
    HttpResponse::Ok()
}
