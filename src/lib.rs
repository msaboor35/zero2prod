pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use routes::{health_check, subscribe};
use sqlx::PgConnection;

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}

pub fn run(port: u16, connection: PgConnection) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .configure(configure_app)
            .app_data(connection.clone())
    })
    .bind(("127.0.0.1", port))?
    .run();

    Ok(server)
}
