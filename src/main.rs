use zero2prod::{configuration::get_configuration, run, startup::init_db};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = get_configuration().expect("Failed to read configuration");
    init_db().await;
    println!("{:?}", config);
    run(config.port)?.await
}
