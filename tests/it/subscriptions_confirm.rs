use actix_http::StatusCode;
use actix_web::test;
use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::{init::TestApp, subscriptions::post_subscription_request};

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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let confirmation_link = get_link(body["Messages"][0]["HTMLPart"].as_str().unwrap());
    let url = Url::parse(&confirmation_link).unwrap();
    let host = url.host_str().unwrap();
    assert_eq!(host, "127.0.0.1"); // make sure the request is going only to local while testing

    let req = test::TestRequest::get()
        .uri(&confirmation_link)
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
