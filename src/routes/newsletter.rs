use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use anyhow::Context;
use sqlx::PgPool;

use crate::{domain::subscriber_email::SubscriberEmail, email_client::EmailClient};

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
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> actix_http::StatusCode {
        match self {
            PublishError::UnexpectedError(_) => actix_http::StatusCode::INTERNAL_SERVER_ERROR,
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
) -> Result<impl Responder, PublishError> {
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
