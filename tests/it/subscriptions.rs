use crate::init::init_app;
use actix_web::{http::StatusCode, test};

use std::vec;

#[actix_web::test]
async fn test_subscribe_returns_200_for_valid_form() {
    let app = init_app().await;

    let conn = zero2prod::startup::DB_POOL.get().unwrap().clone();

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .set_form(form)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(conn.get_ref())
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "test@testdomain.com");
    assert_eq!(saved.name, "Testing tester");
}

#[actix_web::test]
async fn test_subscribe_returns_400_for_incomplete_form() {
    let app = init_app().await;

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

// TODO: Add test for duplicate email
