use actix_web::web;
use sqlx::PgPool;
use std::sync::Once;
use zero2prod::{
    configuration::get_configuration,
    startup::DB_POOL,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Once = Once::new();

fn init_tracing() {
    TRACING.call_once(|| {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    });
}

async fn init_test_db() {
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

pub async fn init() {
    init_tracing();
    init_test_db().await;
}
