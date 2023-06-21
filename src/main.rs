use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration");
    println!("{:?}", config);
    let connection = PgConnection::connect(&config.db.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    run(config.port, connection)?.await
}
