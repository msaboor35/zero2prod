use actix_web::{get, http::header::ContentType, HttpResponse, Responder};

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Home page")]
#[get("/")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home.html"))
}
