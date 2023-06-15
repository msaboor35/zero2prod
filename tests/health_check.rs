use std::vec;

use actix_web::{http::StatusCode, test, App};
use zero2prod::configure_app;

#[actix_web::test]
async fn health_check_test() {
    let app = test::init_service(App::new().configure(configure_app)).await;
    let req = test::TestRequest::get().uri("/health_check").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body.len(), 0);
}

#[actix_web::test]
async fn test_subscribe_returns_200_for_valid_form() {
    let app = test::init_service(App::new().configure(configure_app)).await;
    let form = [("email", "test@testdomain.com"), ("name", "Testing tester")];
    let form = serde_urlencoded::to_string(form)
            .expect("Failed to encode the test case into a query string");
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .set_form(form)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_subscribe_returns_400_for_incomplete_form() {
    let app = test::init_service(App::new().configure(configure_app)).await;

    let test_cases = vec![
        ([None, Some(("name", "Testing tester"))], "missing email"),
        (
            [Some(("email", "test@testdomain.com")), None],
            "missing name",
        ),
        ([None, None], "missing email and password"),
    ];

    for (test_case, error_message) in test_cases {
        let form = serde_urlencoded::to_string(test_case)
            .expect("Failed to encode the test case into a query string");

        let req = test::TestRequest::post()
            .uri("/subscriptions")
            .set_form(&form)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail with 400 Bad Request when the payload was {}",
            &error_message
        );
    }
}
