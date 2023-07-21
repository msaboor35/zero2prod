use actix_http::header::{HeaderMap, HeaderValue};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder, ResponseError};
use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{engine::general_purpose, Engine};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{get_credentials, AuthError, Credentials},
    domain::subscriber_email::SubscriberEmail,
    email_client::EmailClient,
};

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
struct NewsletterBody {
    title: String,
    content: NewsletterContent,
}

#[derive(serde::Deserialize)]
struct NewsletterContent {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_http::body::BoxBody> {
        match self {
            Self::UnexpectedError(_) => {
                HttpResponse::new(actix_http::StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::AuthError(_) => {
                let mut response = HttpResponse::new(actix_http::StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(actix_http::header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Publish newsletter request", skip(body, pool, email_client))]
#[post("/newsletter")]
async fn publish_newsletter(
    body: web::Json<NewsletterBody>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<impl Responder, PublishError> {
    let credentials = basic_auth(request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => email_client
                .send_email(
                    &subscriber.email,
                    &body.title,
                    &body.content.html,
                    &body.content.text,
                )
                .await
                .with_context(|| {
                    format!(
                        "Failed to send email to the subscriber {:}",
                        subscriber.email
                    )
                })?,
            Err(e) => {
                tracing::warn!(e.cause_chain = ?e, "Skipping invalid subscriber: {:?}. The stored email is invalid", e);
            }
        }
    }
    Ok(HttpResponse::Ok())
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Create subscription request", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(e) => Err(anyhow::anyhow!(e)),
        })
        .collect();

    Ok(confirmed_subscribers)
}

fn basic_auth(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The 'Authorization' header was not valid UTF-8 string.")?;

    let base64_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = general_purpose::STANDARD
        .decode(base64_segment)
        .context("Failed to base64 decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credentials were not valid UTF-8 encoded.")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' authorization"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' authorization"))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut id = None;
    let mut expected_password_hash = Secret::new("$argon2id$v=19$m=19456,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".to_string());
    if let Some((stored_id, stored_password_hash)) =
        get_credentials(pool, &credentials.username).await?
    {
        id = Some(stored_id);
        expected_password_hash = stored_password_hash;
    }

    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| verify_password(expected_password_hash, credentials.password))
    })
    .await
    .context("Failed to spawn blocking task")??;

    id.ok_or_else(|| AuthError::InvalidCredentials(anyhow!("Unknown username")))
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_hash)
)]
fn verify_password(
    expected_password_hash: Secret<String>,
    password_hash: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse password hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_hash.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}
