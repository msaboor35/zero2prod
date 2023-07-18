use actix_http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use anyhow::Context;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::subscriber::Subscriber;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::email_client::EmailClient;

#[derive(serde::Deserialize, Debug)]
struct SubscriptionForm {
    name: String,
    email: String,
}

impl TryFrom<SubscriptionForm> for Subscriber {
    type Error = String;

    fn try_from(subscriber: SubscriptionForm) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(subscriber.name)?;
        let email = SubscriberEmail::parse(subscriber.email)?;
        Ok(Self { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
#[post("/subscriptions")]
async fn subscribe(
    form: web::Form<SubscriptionForm>,
    connection: web::Data<PgPool>,
    email_client: web::Data<crate::email_client::EmailClient>,
    base_url: web::Data<crate::startup::ApplicationBaseUrl>,
) -> Result<impl Responder, SubscribeError> {
    let subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = connection
        .begin()
        .await
        .context("Failed to acquire Postgres connection from the pool.")?;
    let subscriber_id = insert_subscription(&mut transaction, &subscriber)
        .await
        .context("Failed to insert a new subscriber into the database.")?;
    let subscription_token = generate_subscription_token();
    store_token_in_db(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store confirmation token in the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    send_confirmation_email(&email_client, subscriber, &base_url.0, &subscription_token)
        .await
        .context("Failed to send the confirmation email to a new subscriber")?;
    Ok(HttpResponse::Ok())
}

#[tracing::instrument(
    name = "Sending a confirmation email",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: Subscriber,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?token={}", base_url, token,);

    let html_content = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    let text_content = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome to the Newsletter",
            &html_content,
            &text_content,
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, transaction)
)]
async fn insert_subscription(
    transaction: &mut PgConnection,
    subscriber: &Subscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, status)
        VALUES ($1, $2, $3, 'pending_confirmation')
        "#,
        subscriber_id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
    )
    .execute(transaction)
    .await?;
    Ok(subscriber_id)
}

#[tracing::instrument(name = "Generating a random subscription token")]
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Storing subscription token in the database",
    skip(transaction, token)
)]
async fn store_token_in_db(
    transaction: &mut PgConnection,
    id: Uuid,
    token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (id, token)
        VALUES ($1, $2)
        "#,
        id,
        token,
    )
    .execute(transaction)
    .await
    .map_err(StoreTokenError)?;
    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while storing a subscription token."
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self.0, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;

    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
