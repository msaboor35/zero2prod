use actix_http::header::HeaderValue;
use actix_web::test;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::{
    helpers::{
        add_test_user, basic_auth, get_confirmation_link, post_newsletter,
        post_subscription_request,
    },
    init::TestApp,
};

#[actix_web::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();
    let pool = app.get_db_conn();

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

    let (_, username, password) = add_test_user(pool).await;
    let auth_header_val = HeaderValue::from_str(&basic_auth(&username, &password)).unwrap();
    let mut req = post_newsletter(&newsletter_body);
    req.headers_mut()
        .insert(actix_http::header::AUTHORIZATION, auth_header_val);

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let email_server = app.get_email_server();
    let pool = app.get_db_conn();

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

    let (_, username, password) = add_test_user(pool).await;
    let auth_header_val = HeaderValue::from_str(&basic_auth(&username, &password)).unwrap();
    let mut req = post_newsletter(&newsletter_body);
    req.headers_mut()
        .insert(actix_http::header::AUTHORIZATION, auth_header_val);

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn newsletter_return_400_for_invalid_data() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let test_cases = vec![
        (serde_json::json!({}), "missing title and content"),
        (
            serde_json::json!({"title": "Newsletter title"}),
            "missing content",
        ),
        (
            serde_json::json!({"content": {"text": "Newsletter content", "html": "<p>Newsletter content</p>"}}),
            "missing title",
        ),
        (
            serde_json::json!(
                {
                    "title": "Newsletter title",
                    "content": {
                        "html": "<p>Newsletter content</p>"
                    }
                }
            ),
            "missing text content",
        ),
        (
            serde_json::json!(
                {
                    "title": "Newsletter title",
                    "content": {
                        "text": "Newsletter content"
                    }
                }
            ),
            "missing html content",
        ),
    ];

    for (body, error) in test_cases {
        let req = post_newsletter(&body);

        let resp = test::call_service(&server, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::BAD_REQUEST,
            "The API did not return with 400 Bad request when the payload was {}",
            error
        );
    }
}

#[actix_web::test]
async fn newsletter_returns_401_when_authorization_headers_are_missing() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>"
        }
    });

    let req = post_newsletter(&newsletter_body);
    let resp = test::call_service(&server, req).await;

    assert_eq!(actix_web::http::StatusCode::UNAUTHORIZED, resp.status());

    let auth_header = resp
        .headers()
        .get("WWW-Authenticate")
        .map(|r| r.to_str().unwrap());
    assert_eq!(Some(r#"Basic realm="publish""#), auth_header);
}

#[actix_web::test]
async fn non_existing_user_is_rejected() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>"
        }
    });

    let username = uuid::Uuid::new_v4().to_string();
    let password = uuid::Uuid::new_v4().to_string();
    let auth_header_val = HeaderValue::from_str(&basic_auth(&username, &password)).unwrap();

    let mut req = post_newsletter(&newsletter_body);
    req.headers_mut()
        .insert(actix_http::header::AUTHORIZATION, auth_header_val);

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    let auth_header = resp
        .headers()
        .get("WWW-Authenticate")
        .map(|r| r.to_str().unwrap());
    assert_eq!(Some(r#"Basic realm="publish""#), auth_header);
}

#[actix_web::test]
async fn invalid_password_is_rejected() {
    let app = TestApp::new().await;
    let server = app.get_server().await;
    let pool = app.get_db_conn();

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>"
        }
    });

    let (_, username, password) = add_test_user(pool).await;

    let invalid_password = uuid::Uuid::new_v4().to_string();
    assert_ne!(password, invalid_password);
    let auth_header_val = HeaderValue::from_str(&basic_auth(&username, &invalid_password)).unwrap();

    let mut req = post_newsletter(&newsletter_body);
    req.headers_mut()
        .insert(actix_http::header::AUTHORIZATION, auth_header_val);

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    let auth_header = resp
        .headers()
        .get("WWW-Authenticate")
        .map(|r| r.to_str().unwrap());
    assert_eq!(Some(r#"Basic realm="publish""#), auth_header);
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
