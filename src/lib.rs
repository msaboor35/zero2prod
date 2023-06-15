use actix_web::dev::Server;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(serde::Deserialize)]
struct SubscriptionForm {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(_form: web::Form<SubscriptionForm>) -> impl Responder {
    HttpResponse::Ok()
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().configure(configure_app))
        .bind(("127.0.0.1", 8080))?
        .run();

    Ok(server)
}
