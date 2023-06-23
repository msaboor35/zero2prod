use crate::init::init_app;
use actix_web::{http::StatusCode, test};

#[actix_web::test]
async fn health_check_test() {
    let app = init_app().await;
    let req = test::TestRequest::get().uri("/health_check").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body.len(), 0);
}
