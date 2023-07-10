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
            http_client: Client::new(),
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
                    email: self.sender.as_ref().to_owned(),
                },
                to: vec![SendEmailRequestEmail {
                    email: recipient.as_ref().to_owned(),
                }],
                subject: subject.to_owned(),
                text_part: text_content.to_owned(),
                html_part: html_content.to_owned(),
            }],
        };

        println!("{}", serde_json::to_string_pretty(&request_body).unwrap());

        self
            .http_client
            .post(url)
            .basic_auth(
                self.api_key.expose_secret(),
                Some(self.api_secret.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestEmail {
    email: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestMessage {
    from: SendEmailRequestEmail,
    to: Vec<SendEmailRequestEmail>,
    subject: String,
    text_part: String,
    html_part: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    messages: Vec<SendEmailRequestMessage>,
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::{domain::subscriber_email::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let api_key = Secret::new(Faker.fake::<String>());
        let api_secret = Secret::new(Faker.fake::<String>());
        let email_client = EmailClient::new(mock_server.uri(), sender, api_key, api_secret);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Sentence(1..2).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
