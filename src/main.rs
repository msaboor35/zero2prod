use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(configure_app))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
