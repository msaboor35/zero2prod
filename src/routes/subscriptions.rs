use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
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

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection, email_client),
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
) -> impl Responder {
    let subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest(),
    };

    if insert_subscription(&subscriber, &connection).await.is_err() {
        return HttpResponse::InternalServerError();
    }

    if send_confirmation_email(&email_client, subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError();
    }

    HttpResponse::Ok()
}

#[tracing::instrument(
    name = "Sending a confirmation email",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: Subscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        "http://localhost:8080", // TODO: get this from the configuration
        "subscription_token",    // TODO: generate a unique token
    );

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
    skip(subscriber, connection)
)]
async fn insert_subscription(
    subscriber: &Subscriber,
    connection: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, status)
        VALUES ($1, $2, $3, 'confirmed')
        "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
