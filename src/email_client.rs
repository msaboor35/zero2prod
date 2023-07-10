use crate::domain::subscriber_email::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    api_key: Secret<String>,
    api_secret: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        api_key: Secret<String>,
        api_secret: Secret<String>,
    ) -> Self {
        Self {
            sender,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            base_url,
            api_key,
            api_secret,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3.1/send", self.base_url);

        let request_body = SendEmailRequest {
            messages: vec![SendEmailRequestMessage {
                from: SendEmailRequestEmail {
                    email: self.sender.as_ref(),
                },
                to: vec![SendEmailRequestEmail {
                    email: recipient.as_ref(),
                }],
                subject,
                text_part: text_content,
                html_part: html_content,
            }],
        };

        println!("{}", serde_json::to_string_pretty(&request_body).unwrap());

        self.http_client
            .post(url)
            .basic_auth(
                self.api_key.expose_secret(),
                Some(self.api_secret.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestEmail<'a> {
    email: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestMessage<'a> {
    from: SendEmailRequestEmail<'a>,
    to: Vec<SendEmailRequestEmail<'a>>,
    subject: &'a str,
    text_part: &'a str,
    #[serde(rename = "HTMLPart")]
    html_part: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    messages: Vec<SendEmailRequestMessage<'a>>,
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
        Fake, Faker,
    };
    use secrecy::{ExposeSecret, Secret};
    use wiremock::{
        matchers::{any, basic_auth, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{domain::subscriber_email::SubscriberEmail, email_client::EmailClient};

    struct SendEmailRequestBodyMatcher;

    impl wiremock::Match for SendEmailRequestBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            result
                .as_ref()
                .ok()
                .and_then(|body| {
                    body.get("Messages")
                        .and_then(|messages| messages.as_array().filter(|arr| arr.len() == 1))
                        .and_then(|messages| messages.get(0))
                        .and_then(|message| {
                            message
                                .get("From")
                                .and_then(|from| from.get("Email"))
                                .and_then(|from_email| from_email.as_str())
                                .and_then(|_| {
                                    message
                                        .get("To")
                                        .and_then(|to| to.as_array().filter(|arr| arr.len() == 1))
                                        .and_then(|to| to.get(0))
                                        .and_then(|to| to.get("Email"))
                                        .and_then(|to_email| to_email.as_str())
                                })
                                .and_then(|_| {
                                    message.get("Subject").and_then(|subject| subject.as_str())
                                })
                                .and_then(|_| {
                                    message
                                        .get("TextPart")
                                        .and_then(|text_part| text_part.as_str())
                                })
                                .and_then(|_| {
                                    message
                                        .get("HTMLPart")
                                        .and_then(|html_part| html_part.as_str())
                                })
                        })
                })
                .is_some()
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header("Content-Type", "application/json"))
            .and(basic_auth(
                email_client.api_key.expose_secret(),
                email_client.api_secret.expose_secret(),
            ))
            .and(path("/v3.1/send"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Sentence(1..2).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        let api_key = Secret::new(Faker.fake::<String>());
        let api_secret = Secret::new(Faker.fake::<String>());
        EmailClient::new(base_url, email(), api_key, api_secret)
    }
}
