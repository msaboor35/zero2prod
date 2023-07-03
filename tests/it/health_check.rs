use crate::init::TestApp;
use actix_web::{http::StatusCode, test};

#[actix_web::test]
async fn health_check_test() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let req = test::TestRequest::get().uri("/health_check").to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body.len(), 0);
}
