use crate::configuration::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::newsletter::publish_newsletter;
use crate::routes::{confirm, health_check, home, login, login_form, subscribe};
use actix_http::body::BoxBody;
use actix_web::cookie::Key;
use actix_web::dev::{Server, ServiceFactory, ServiceResponse};
use actix_web::web::{Data, ServiceConfig};
use actix_web::{App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use tracing_actix_web::{StreamSpan, TracingLogger};

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

#[derive(Clone, Debug)]
pub struct HmacSecret(pub Secret<String>);

pub struct Application {
    config: Settings,
    server: Server,
}

impl Application {
    pub async fn new(config: Settings) -> Result<Self, std::io::Error> {
        let db_pool = init_db(&config.db);
        let email_client = init_email_client(&config.email_client);
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

pub fn init_email_client(config: &EmailClientSettings) -> EmailClient {
    let sender_email = config.sender().expect("Invalid sender email address");
    EmailClient::new(
        config.base_url.clone(),
        sender_email,
        config.api_key.clone(),
        config.api_secret.clone(),
        config.timeout(),
    )
}

pub fn new_app(
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
    base_url: Data<ApplicationBaseUrl>,
    hmac_secret: HmacSecret,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = ServiceResponse<StreamSpan<BoxBody>>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let message_store =
        CookieMessageStore::builder(Key::from(hmac_secret.0.expose_secret().as_bytes())).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    App::new()
        .wrap(message_framework)
        .wrap(TracingLogger::default())
        .configure(configure_app)
        .app_data(pool)
        .app_data(email_client)
        .app_data(base_url)
}

fn run(
    config: &ApplicationSettings,
    pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let pool = Data::new(pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(config.base_url.clone()));
    let hmac_secret = HmacSecret(config.hmac_secret.clone());

    let server = HttpServer::new(move || {
        new_app(
            pool.clone(),
            email_client.clone(),
            base_url.clone(),
            hmac_secret.clone(),
        )
    })
    .bind((config.host.clone(), config.port))?
    .run();

    Ok(server)
}

pub fn configure_app(cfg: &mut ServiceConfig) {
    cfg.service(health_check)
        .service(subscribe)
        .service(confirm)
        .service(publish_newsletter)
        .service(home)
        .service(login_form)
        .service(login);
}
