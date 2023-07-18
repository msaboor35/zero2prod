use actix_web::test;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::{
    helpers::{get_confirmation_link, post_subscription_request},
    init::TestApp,
};

#[actix_web::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(email_server)
        .await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>"
        }
    });

    let req = test::TestRequest::post()
        .uri("/newsletter")
        .set_json(newsletter_body)
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    create_confirmed_subscriber(&app).await;

    Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(email_server)
        .await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>"
        }
    });

    let req = test::TestRequest::post()
        .uri("/newsletter")
        .set_json(newsletter_body)
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> String {
    let server = app.get_server().await;
    let email_server = app.get_email_server();

    let _mock_guard = Mock::given(path("/v3.1/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount_as_scoped(email_server)
        .await;

    let form = &[("email", "test@testdomain.com"), ("name", "Testing tester")];
    let req = post_subscription_request(form);

    test::call_service(&server, req).await;
    let email_request = &email_server.received_requests().await.unwrap()[0];
    get_confirmation_link(&email_request.body)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    let server = app.get_server().await;

    let req = test::TestRequest::get()
        .uri(&confirmation_link)
        .to_request();

    test::call_service(&server, req).await;
}
