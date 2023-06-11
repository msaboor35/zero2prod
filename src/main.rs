use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(configure_app))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use crate::configure_app;
    use actix_web::{http::StatusCode, test, App};

    #[actix_web::test]
    async fn health_check_test() {
        let app = test::init_service(App::new().configure(configure_app)).await;
        let req = test::TestRequest::get().uri("/health_check").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
