//! src/main.rs

use secrecy::ExposeSecret;

use emailer::{
  configuration::get_configuration,
  startup::run,
  telementry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;
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
  let connection_pool = PgPool::connect_lazy(
    &configuration.database.connection_string().expose_secret(),
  )
  .expect("Failed to connect to Postgres.");

  let address = format!(
    "{}:{}",
    configuration.application.host, configuration.application.port
  );
  let listener = TcpListener::bind(address.clone())?;

  run(listener, connection_pool)?.await?;
  Ok(())
}
