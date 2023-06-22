use zero2prod::{
    configuration::get_configuration,
    run,
    startup::init_db,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");
    init_db().await;
    println!("{:?}", config);
    run(config.port)?.await
}
