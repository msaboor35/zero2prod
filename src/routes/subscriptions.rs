use actix_web::{post, web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
struct SubscriptionForm {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(_form: web::Form<SubscriptionForm>) -> impl Responder {
    HttpResponse::Ok()
}
