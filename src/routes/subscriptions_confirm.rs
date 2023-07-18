use actix_http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
}

#[derive(thiserror::Error)]
pub enum SubscribtionConfirmError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no subscriber with the given token")]
    InvalidToken,
}

impl std::fmt::Debug for SubscribtionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribtionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
        }
    }
}

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, connection))]
#[get("/subscriptions/confirm")]
async fn confirm(
    parameters: web::Query<Parameters>,
    connection: web::Data<PgPool>,
) -> Result<impl Responder, SubscribtionConfirmError> {
    let id = get_subscriber_id_from_token(&connection, &parameters.token)
        .await
        .context("Failed to retrive the subscriber id for the given token.")?
        .ok_or(SubscribtionConfirmError::InvalidToken)?;
    confirm_subscriber(&connection, id)
        .await
        .context("Failed to change the status of the subscriber to `confirmed`.")?;
    Ok(HttpResponse::Ok())
}

#[tracing::instrument(
    name = "Storing new subscriber details in the database",
    skip(id, pool)
)]
async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Getting subscriber_id from token", skip(pool, token))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id FROM subscription_tokens WHERE token = $1"#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.id))
}
