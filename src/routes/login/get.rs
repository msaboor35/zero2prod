use actix_web::{get, http::header::ContentType, HttpResponse, Responder};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

#[allow(clippy::async_yields_async, clippy::let_with_type_underscore)]
#[tracing::instrument(name = "Serve login page", skip(flash_message))]
#[get("/login")]
pub async fn login_form(flash_message: IncomingFlashMessages) -> impl Responder {
    let mut error_html = String::new();
    for message in flash_message.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", message.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Home</title>
    </head>
    <body>
        {error_html}
        <form action="/login" method="post">
            <label>
                Username
                <input type="text" name="username" placeholder="Enter username">
            </label>
            <label>
                Password
                <input type="password" name="password" placeholder="Enter password">
            </label>

            <button type="submit">Login</button>
        </body>
    </form>
</html>
            "#,
        ))
}
