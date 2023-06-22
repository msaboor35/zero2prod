use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
struct SubscriptionForm {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(
    form: web::Form<SubscriptionForm>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let request_id = uuid::Uuid::new_v4();

    let request_span = tracing::info_span!("Adding a new subscriber", %request_id, email = %form.email, name = %form.name);
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("{} - Saved new subscriber details.", request_id);
            HttpResponse::Ok()
        }
        Err(e) => {
            tracing::error!("{} - Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError()
        }
    }
}
