use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
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

    id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Unknown username")))
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_hash)
)]
pub fn verify_password(
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

#[tracing::instrument(name = "Get stored credentials", skip(pool))]
pub async fn get_credentials(
    pool: &PgPool,
    username: &str,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform query to retrieve stored credentials.")?
    .map_or_else(
        || None,
        |row| Some((row.id, Secret::new(row.password_hash))),
    );

    Ok(row)
}
