use crate::init::init;
use actix_web::{http::StatusCode, test, App};
use zero2prod::startup::configure_app;

#[actix_web::test]
async fn health_check_test() {
    init().await;
    let app = test::init_service(App::new().configure(configure_app)).await;
    let req = test::TestRequest::get().uri("/health_check").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body.len(), 0);
}
