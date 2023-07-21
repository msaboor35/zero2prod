use actix_web::{get, http::header::ContentType, HttpResponse, Responder};

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Serve login page")]
#[get("/login")]
pub async fn login_form() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}
