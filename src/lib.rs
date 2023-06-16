pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use routes::{health_check, subscribe};

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().configure(configure_app))
        .bind(("127.0.0.1", 8080))?
        .run();

    Ok(server)
}
