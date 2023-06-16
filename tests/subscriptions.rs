use actix_web::{http::StatusCode, test, App};
use zero2prod::configure_app;

use std::vec;

#[actix_web::test]
async fn test_subscribe_returns_200_for_valid_form() {
    let app = test::init_service(App::new().configure(configure_app)).await;
    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
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
        (
            [("email", None), ("name", Some("Testing tester"))],
            "missing email",
        ),
        (
            [("email", Some("test@testdomain.com")), ("name", None)],
            "missing name",
        ),
        (
            [("email", None), ("name", None)],
            "missing email and password",
        ),
    ];

    for (test_case, error_message) in test_cases {
        let form = test_case.as_slice();

        let req = test::TestRequest::post()
            .uri("/subscriptions")
            .set_form(form)
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
