use actix_web::{post, HttpResponse, Responder};

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Publish newsletter request")]
#[post("/newsletter")]
async fn publish_newsletter() -> impl Responder {
    HttpResponse::Ok()
}
