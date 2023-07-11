use crate::{
    helpers::{get_confirmation_link, post_subscription_request},
    init::TestApp,
};
use actix_web::{http::StatusCode, test};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use std::vec;

#[actix_web::test]
async fn test_subscribe_returns_200_for_valid_form() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(email_server)
        .await;

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_subscribe_persists_the_new_subscriber() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(email_server)
        .await;

    let conn = app.get_db_conn();
    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    test::call_service(&server, req).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(conn)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "test@testdomain.com");
    assert_eq!(saved.name, "Testing tester");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_web::test]
async fn test_subscribe_returns_400_for_incomplete_form() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let test_cases = vec![
        (
            [("email", None), ("name", Some("Testing tester 1"))],
            "missing email",
        ),
        (
            [("email", Some("test1@testdomain.com")), ("name", None)],
            "missing name",
        ),
        (
            [("email", None), ("name", None)],
            "missing email and password",
        ),
    ];

    for (test_case, error_message) in test_cases {
        let form = test_case.as_slice();

        let req = post_subscription_request(form);

        let resp = test::call_service(&server, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail with 400 Bad Request when the payload was {}",
            &error_message
        );
    }
}

#[actix_web::test]
async fn test_subscribe_returns_400_when_data_is_missing() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let test_cases = vec![
        ([("email", ""), ("name", "Testing tester2")], "empty email"),
        (
            [("email", "test2@testdomain.com"), ("name", "")],
            "empty name",
        ),
        ([("email", ""), ("name", "")], "empty email and password"),
    ];

    for (test_case, error_message) in test_cases {
        let form = test_case.as_slice();

        let req = post_subscription_request(form);

        let resp = test::call_service(&server, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail with 400 Bad Request when the payload was {}",
            &error_message
        );
    }
}

#[actix_web::test]
async fn test_subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(email_server)
        .await;

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    test::call_service(&server, req).await;
}

#[actix_web::test]
async fn test_subscribe_sends_a_confirmation_email_with_a_link() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(email_server)
        .await;

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    test::call_service(&server, req).await;

    let email_request = &email_server.received_requests().await.unwrap()[0];
    get_confirmation_link(&email_request.body);
}

// TODO: Add test for duplicate email
