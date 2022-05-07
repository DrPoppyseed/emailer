//! src/main.rs

use emailer::{
  configuration::get_configuration,
  email_client::EmailClient,
  startup::run,
  telementry::{get_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;
use std::{
  io::{stdout, Result},
  net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
  let subscriber = get_subscriber("emailer".into(), "info".into(), stdout);
  init_subscriber(subscriber);

  let configuration =
    get_configuration().expect("Failed to read configuration.");
  let connection_pool = PgPoolOptions::new()
    .connect_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(configuration.database.with_db());

  let timeout = configuration.email_client.timeout();

  let sender_email = configuration
    .email_client
    .sender()
    .expect("Invalid sender email address.");
  let email_client = EmailClient::new(
    configuration.email_client.base_url,
    sender_email,
    configuration.email_client.authorization_token,
    timeout,
  );

  let address = format!(
    "{}:{}",
    configuration.application.host, configuration.application.port
  );
  let listener = TcpListener::bind(address.clone())?;

  run(listener, connection_pool, email_client)?.await?;
  Ok(())
}
