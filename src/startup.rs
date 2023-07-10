use crate::configuration::{ApplicationSettings, DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub struct Application {
    config: Settings,
    server: Server,
}

impl Application {
    pub async fn new(config: Settings) -> Result<Self, std::io::Error> {
        let db_pool = init_db(&config.db);
        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let email_client = EmailClient::new(
            config.email_client.base_url.clone(),
            sender_email,
            config.email_client.api_key.clone(),
            config.email_client.api_secret.clone(),
        );
        let server = run(&config.app, db_pool, email_client).expect("Failed to bind address");
        let app = Application { config, server };
        Ok(app)
    }

    pub fn get_config(&self) -> &Settings {
        &self.config
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn init_db(config: &DatabaseSettings) -> PgPool {
    PgPool::connect_lazy_with(config.with_db())
}

fn run(
    config: &ApplicationSettings,
    pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let pool = Data::new(pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(configure_app)
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .bind((config.host.clone(), config.port))?
    .run();

    Ok(server)
}

pub fn configure_app(cfg: &mut ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}
