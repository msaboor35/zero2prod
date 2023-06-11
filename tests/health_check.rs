use actix_web::{http::StatusCode, test, App};
use zero2prod::configure_app;

#[actix_web::test]
async fn health_check_test() {
    let app = test::init_service(App::new().configure(configure_app)).await;
    let req = test::TestRequest::get().uri("/health_check").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
