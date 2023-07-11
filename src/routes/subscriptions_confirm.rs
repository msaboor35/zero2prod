use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, connection))]
#[get("/subscriptions/confirm")]
async fn confirm(
    parameters: web::Query<Parameters>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let id = match get_subscriber_id_from_token(&connection, &parameters.token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError(),
    };

    match id {
        None => HttpResponse::Unauthorized(),
        Some(subscriber_id) => {
            if confirm_subscriber(&connection, subscriber_id)
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError();
            }
            HttpResponse::Ok()
        }
    }
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.id))
}
