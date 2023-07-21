use actix_web::http::header;
use actix_web::{post, HttpResponse, Responder};

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Process login request")]
#[post("/login")]
pub async fn login() -> impl Responder {
    let mut resp = HttpResponse::SeeOther();
    resp.insert_header((header::LOCATION, "/"));
    resp
}
