pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::dev::Server;
use actix_web::{App, HttpServer};
use startup::configure_app;

pub fn run(port: u16) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || App::new().configure(configure_app))
        .bind(("127.0.0.1", port))?
        .run();

    Ok(server)
}
