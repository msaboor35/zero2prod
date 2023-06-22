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
    log::info!("Received subscription request");
    log::info!("Adding '{}' '{}' to database", form.email, form.name);
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
            log::info!("Saved new subscriber details.");
            HttpResponse::Ok()
        }
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError()
        }
    }
}
