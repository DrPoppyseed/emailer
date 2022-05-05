//! src/main.rs

use emailer::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use std::{
  io::{stdout, Result},
  net::TcpListener,
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
  // Redirect all log events to our subscriber
  LogTracer::init().expect("Failed to set logger");

  // Print all spans at info-level or above if RUST_LOG env var has not been set.
  let env_filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info"));
  let formatting_layer = BunyanFormattingLayer::new(
    "emailer".into(),
    // Output the formatted spans to stdout
    stdout,
  );

  let subscriber = Registry::default()
    .with(env_filter)
    .with(JsonStorageLayer)
    .with(formatting_layer);
  set_global_default(subscriber).expect("Failed to set subscriber");

  let configuration =
    get_configuration().expect("Failed to read configuration.");
  let connection_pool =
    PgPool::connect(&configuration.database.connection_string())
      .await
      .expect("Failed to connect to Postgres.");
  let address = format!("127.0.0.1:{}", configuration.application_port);
  let listener = TcpListener::bind(address.clone())?;

  run(listener, connection_pool)?.await
}
