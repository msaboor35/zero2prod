use actix_http::Request;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{body::BoxBody, test, web, App};
use sqlx::PgPool;
use std::sync::Once;
use tracing_actix_web::{StreamSpan, TracingLogger};
use zero2prod::startup::configure_app;
use zero2prod::{
    configuration::get_configuration,
    startup::DB_POOL,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Once = Once::new();

pub async fn init_app(
) -> impl Service<Request, Response = ServiceResponse<StreamSpan<BoxBody>>, Error = actix_web::Error>
{
    init().await;

    test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(configure_app),
    )
    .await
}

fn init_tracing() {
    TRACING.call_once(|| {
        let subscriber_name = "test".into();
        let log_level = "debug".into();

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, log_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, log_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });
}

async fn init_test_db() {
    use sqlx::{Connection, Executor, PgConnection};
    use uuid::Uuid;

    let mut config = get_configuration().expect("Failed to read configuration");
    config.db.name = Uuid::new_v4().to_string();
    let mut conn = PgConnection::connect_with(&config.db.without_db())
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.db.name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = PgPool::connect_with(config.db.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let pool = web::Data::new(pool);
    _ = DB_POOL.set(pool);
}

async fn init() {
    init_tracing();
    init_test_db().await;
}
