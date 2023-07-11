use actix_http::StatusCode;
use actix_web::test;
use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::{helpers::get_confirmation_link, helpers::post_subscription_request, init::TestApp};

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

#[actix_web::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
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
    let confirmation_link = get_confirmation_link(&email_request.body);
    let url = Url::parse(&confirmation_link).unwrap();
    let host = url.host_str().unwrap();
    assert_eq!(host, "127.0.0.1"); // make sure the request is going only to local while testing

    let req = test::TestRequest::get()
        .uri(&confirmation_link)
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn clicking_on_confirmation_link_confirms_a_subscriber() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();
    let conn = app.get_db_conn();

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(email_server)
        .await;

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    test::call_service(&server, req).await;

    let email_request = &email_server.received_requests().await.unwrap()[0];
    let confirmation_link = get_confirmation_link(&email_request.body);
    let url = Url::parse(&confirmation_link).unwrap();
    let host = url.host_str().unwrap();
    assert_eq!(host, "127.0.0.1"); // make sure the request is going only to local while testing

    let req = test::TestRequest::get()
        .uri(&confirmation_link)
        .to_request();

    test::call_service(&server, req).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(conn)
        .await
        .unwrap();

    assert_eq!(saved.email, "test@testdomain.com");
    assert_eq!(saved.name, "Testing tester");
    assert_eq!(saved.status, "confirmed");
}
