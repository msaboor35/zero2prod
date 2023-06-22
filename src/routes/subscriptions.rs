use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
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

    log::info!("{} - Received subscription request", request_id);
    log::info!("{} - Adding '{}' '{}' to database", request_id, form.email, form.name);
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
    .await
    {
        Ok(_) => {
            log::info!("{} - Saved new subscriber details.", request_id);
            HttpResponse::Ok()
        }
        Err(e) => {
            log::error!("{} - Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError()
        }
    }
}
