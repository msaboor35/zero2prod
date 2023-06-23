use crate::configuration::get_configuration;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use std::sync::OnceLock;

pub static DB_POOL: OnceLock<web::Data<PgPool>> = OnceLock::new();

pub fn run(port: u16) -> Result<Server, std::io::Error> {
    let server =
        HttpServer::new(move || App::new().wrap(Logger::default()).configure(configure_app))
            .bind(("127.0.0.1", port))?
            .run();

    Ok(server)
}

pub async fn init_test_db() {
    use sqlx::{Connection, Executor, PgConnection};
    use uuid::Uuid;

    let mut config = get_configuration().expect("Failed to read configuration");
    config.db.name = Uuid::new_v4().to_string();
    let mut conn = PgConnection::connect(&config.db.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.db.name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = PgPool::connect(&config.db.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let pool = web::Data::new(pool);
    _ = DB_POOL.set(pool);
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    let pool = DB_POOL.get().unwrap().clone();
    cfg.service(health_check).service(subscribe).app_data(pool);
}
