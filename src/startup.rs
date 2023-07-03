use crate::configuration::{ApplicationSettings, DatabaseSettings, Settings};
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
        let server = run(&config.app, db_pool).expect("Failed to bind address");
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

pub fn run(config: &ApplicationSettings, pool: PgPool) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(configure_app)
            .app_data(Data::new(pool.clone()))
    })
    .bind((config.host.clone(), config.port))?
    .run();

    Ok(server)
}

pub fn configure_app(cfg: &mut ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}
