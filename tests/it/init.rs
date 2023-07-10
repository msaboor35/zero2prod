use actix_http::Request;
use actix_web::dev::{Service, ServiceFactory, ServiceResponse};
use actix_web::web::Data;
use actix_web::{body::BoxBody, test, App};
use sqlx::PgPool;
use std::sync::Once;
use tracing_actix_web::{StreamSpan, TracingLogger};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::{self, EmailClient};
use zero2prod::startup::{configure_app, init_db, init_email_client, new_app};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Once = Once::new();

pub struct TestApp {
    db_pool: PgPool,
    email_client: EmailClient,
    // server: Box<dyn Service<Request, Response = ServiceResponse<StreamSpan<BoxBody>>, Error = actix_web::Error, Future = Box<dyn Future<Output = Result<ServiceResponse<StreamSpan<BoxBody>>, actix_web::Error>>>>>,
}

impl TestApp {
    pub async fn new() -> Self {
        init_tracing();

        let mut config = get_configuration().expect("Failed to read configuration");
        config.db.name = Uuid::new_v4().to_string();
        config.app.port = 0;

        init_test_db(&config.db).await;

        let db_pool = init_db(&config.db);
        let email_client = init_email_client(&config.email_client);

        TestApp {
            db_pool,
            email_client,
        }
    }

    // TODO: this should be called only once in the new function and the return value should be stored in a field
    // FnOnce Maybe?????
    pub async fn get_server(
        &self,
    ) -> impl Service<Request, Response = ServiceResponse<StreamSpan<BoxBody>>, Error = actix_web::Error>
    {
        let db_pool = Data::new(self.db_pool.clone());
        let email_client = Data::new(self.email_client.clone());
        test::init_service(new_app(db_pool, email_client)).await
    }

    pub fn get_db_conn(&self) -> &PgPool {
        &self.db_pool
    }
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

async fn init_test_db(config: &DatabaseSettings) {
    use sqlx::{Connection, Executor, PgConnection};

    let mut conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");
}
