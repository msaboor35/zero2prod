use actix_web::{post, HttpResponse, Responder};

#[derive(serde::Deserialize)]
struct NewsletterBody {
    title: String,
    content: NewsletterContent,
}

#[derive(serde::Deserialize)]
struct NewsletterContent {
    html: String,
    text: String,
}

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Publish newsletter request", skip(_body))]
#[post("/newsletter")]
async fn publish_newsletter(_body: actix_web::web::Json<NewsletterBody>) -> impl Responder {
    HttpResponse::Ok()
}
