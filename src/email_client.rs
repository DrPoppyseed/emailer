use std::time::Duration;

use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Clone)]
pub struct EmailClient {
  http_client: Client,
  base_url: String,
  sender: SubscriberEmail,
  api_key: Secret<String>,
}

impl EmailClient {
  pub fn new(
    base_url: String,
    sender: SubscriberEmail,
    api_key: Secret<String>,
    timeout: Duration,
  ) -> Self {
    let http_client = Client::builder().timeout(timeout).build().unwrap();
    Self {
      http_client,
      base_url,
      sender,
      api_key,
    }
  }

  pub async fn send_email(
    &self,
    recipient: SubscriberEmail,
    subject: &str,
    html_content: &str,
    text_content: &str,
  ) -> Result<(), reqwest::Error> {
    let url = format!("{}/messages", self.base_url);
    let request_body = SendEmailMessageRequest {
      key: self.api_key.expose_secret(),
      message: SendEmailMessage {
        from_email: self.sender.as_ref(),
        to: vec![SendEmailMessageRecipient {
          email: recipient.as_ref(),
        }],
        subject,
        html: html_content,
        text: text_content,
      },
    };
    self
      .http_client
      .post(&url)
      .json(&request_body)
      .send()
      .await?
      .error_for_status()?;
    Ok(())
  }
}

#[derive(Serialize)]
struct SendEmailMessage<'a> {
  from_email: &'a str,
  to: Vec<SendEmailMessageRecipient<'a>>,
  subject: &'a str,
  html: &'a str,
  text: &'a str,
}

#[derive(Serialize)]
struct SendEmailMessageRecipient<'a> {
  email: &'a str,
}

#[derive(Serialize)]
struct SendEmailMessageRequest<'a> {
  key: &'a str,
  message: SendEmailMessage<'a>,
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use claim::assert_ok;
  use fake::{
    faker::{
      internet::en::SafeEmail,
      lorem::{en::Paragraph, en::Sentence},
    },
    Fake, Faker,
  };
  use secrecy::Secret;
  use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
  };

  use crate::domain::SubscriberEmail;

  use super::EmailClient;

  struct SendEmailBodyMatcher;

  impl wiremock::Match for SendEmailBodyMatcher {
    fn matches(&self, request: &wiremock::Request) -> bool {
      let result: Result<serde_json::Value, _> =
        serde_json::from_slice(&request.body);

      if let Ok(body) = result {
        dbg!(&body);
        body.get("key").is_some() && body.get("message").is_some()
      } else {
        false
      }
    }
  }

  /// Generate a random email subject
  fn subject() -> String {
    Sentence(1..2).fake()
  }

  /// Generate some random email content
  fn content() -> String {
    Paragraph(1..10).fake()
  }

  /// Generate a random subscriber email
  fn email() -> SubscriberEmail {
    SubscriberEmail::parse(SafeEmail().fake()).unwrap()
  }

  /// Get a test instance of `EmailClient`
  fn email_client(base_url: String) -> EmailClient {
    EmailClient::new(
      base_url,
      email(),
      Secret::new(Faker.fake()),
      Duration::from_millis(200),
    )
  }

  #[tokio::test]
  async fn send_email_sends_the_expected_request() {
    // Arrange
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(header("Content-Type", "application/json"))
      .and(path("/messages"))
      .and(method("POST"))
      .and(SendEmailBodyMatcher)
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    // Act
    let outcome = email_client
      .send_email(email(), &subject(), &content(), &content())
      .await;

    assert_ok!(outcome);
  }
}
