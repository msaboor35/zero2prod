use actix_http::body::to_bytes;
use actix_web::{http, test};

use crate::{
    helpers::{assert_is_redirect_to, post_login},
    init::TestApp,
};

#[actix_web::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = TestApp::new().await;
    let server = app.get_server().await;

    // Try to login with invalid credentials
    let login_body = serde_json::json!({
        "username": "nonexistent",
        "password": "password",
    });

    let req = post_login(login_body);
    let resp = test::call_service(&server, req).await;
    let cookie = resp.response().cookies().next().unwrap();

    assert_is_redirect_to(&resp, "/login");

    // Follow the redirect and check that the flash message is displayed
    let login_form_req = test::TestRequest::get()
        .uri("/login")
        .cookie(cookie)
        .to_request();

    let login_form_resp = test::call_service(&server, login_form_req).await;
    let status = login_form_resp.status();
    let login_html = to_bytes(login_form_resp.into_body()).await.unwrap();
    let login_html = std::str::from_utf8(login_html.as_ref()).unwrap();

    assert_eq!(status, http::StatusCode::OK);
    assert!(login_html.contains(r#"<p><i>Invalid credentials</i></p>"#));
}
