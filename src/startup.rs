use crate::configuration::{get_configuration, ApplicationSettings};
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web;
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use std::sync::OnceLock;
use tracing_actix_web::TracingLogger;

pub static DB_POOL: OnceLock<web::Data<PgPool>> = OnceLock::new();

pub fn run(config: &ApplicationSettings) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(configure_app)
    })
    .bind((config.host.clone(), config.port))?
    .run();

    Ok(server)
}

pub async fn init_db() {
    let config = get_configuration().expect("Failed to read configuration");
    let pool = PgPool::connect_lazy_with(config.db.with_db());
    let pool = web::Data::new(pool);
    _ = DB_POOL.set(pool);
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    let pool = DB_POOL.get().unwrap().clone();
    cfg.service(health_check).service(subscribe).app_data(pool);
}
