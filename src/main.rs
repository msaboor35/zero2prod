use zero2prod::{run, configuration::get_configuration};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration");
    println!("{:?}", config);
    run(config.port)?.await
}
