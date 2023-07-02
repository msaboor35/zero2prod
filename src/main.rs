use zero2prod::{
    configuration::get_configuration,
    startup::{init_db, run},
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");
    init_db().await;
    println!("{:?}", config);
    run(&config.app)?.await
}
