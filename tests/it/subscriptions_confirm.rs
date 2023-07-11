use actix_http::StatusCode;
use actix_web::test;

use crate::init::TestApp;

#[actix_web::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let req = test::TestRequest::get()
        .uri("/subscriptions/confirm")
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "The API did not fail with 400 Bad Request when the token was missing"
    );
}
