use actix_web::{App, HttpServer, HttpResponse, Responder, web, get};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(configure_app))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}