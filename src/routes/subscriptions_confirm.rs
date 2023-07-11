use actix_web::{get, web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
#[get("/subscriptions/confirm")]
async fn confirm(_parameters: web::Query<Parameters>) -> impl Responder {
    HttpResponse::Ok()
}
