use actix_http::Request;
use actix_web::test;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::{engine::general_purpose, Engine};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

pub fn get_confirmation_link(body: &[u8]) -> String {
    let body: serde_json::Value = serde_json::from_slice(body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(body["Messages"][0]["HTMLPart"].as_str().unwrap());
    let text_link = get_link(body["Messages"][0]["TextPart"].as_str().unwrap());

    assert_eq!(html_link, text_link);
    html_link
}

pub fn post_subscription_request(form: impl Serialize) -> Request {
    test::TestRequest::post()
        .uri("/subscriptions")
        .set_form(form)
        .to_request()
}

pub fn post_newsletter(body: &serde_json::Value) -> Request {
    test::TestRequest::post()
        .uri("/newsletter")
        .set_json(body)
        .to_request()
}

pub fn basic_auth(username: &str, password: &str) -> String {
    let auth = format!("{}:{}", username, password);
    let encoded = general_purpose::STANDARD.encode(auth.as_bytes());
    format!("Basic {}", encoded)
}

pub async fn add_test_user(pool: &PgPool) -> (Uuid, String, String) {
    let id = Uuid::new_v4();
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash)
        VALUES ($1, $2, $3)
        "#,
        id,
        username,
        password_hash,
    )
    .execute(pool)
    .await
    .expect("Failed to insert test user");

    (id, username, password)
}
