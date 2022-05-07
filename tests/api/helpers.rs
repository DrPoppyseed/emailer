use emailer::{
  configuration::{get_configuration, DatabaseSettings},
  email_client::EmailClient,
  startup::run,
  telementry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{
  env::var,
  io::{sink, stdout},
  net::TcpListener,
};
use tokio::spawn;
use uuid::Uuid;
// Ensures that the `tracing` stack is only initialized once using cargo `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
  let default_filter_level = "info".to_string();
  let subscriber_name = "test".to_string();

  if var("TEST_LOG").is_ok() {
    let subscriber =
      get_subscriber(subscriber_name, default_filter_level, stdout);
    init_subscriber(subscriber);
  } else {
    let subscriber =
      get_subscriber(subscriber_name, default_filter_level, sink);
    init_subscriber(subscriber);
  }
});

pub struct TestApp {
  pub address: String,
  pub db_pool: PgPool,
}

/// Spin up an instance of our application and returns its address
/// (i.e. http://localhost:XXXX)
/// Also spins up a logical database each spawn, to insure the test's isolation
pub async fn spawn_app() -> TestApp {
  // `TRACING` is executed only once: the first time.
  Lazy::force(&TRACING);

  let listener =
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
  let port = listener.local_addr().unwrap().port();
  let address = format!("http://127.0.0.1:{}", port);

  let mut configuration =
    get_configuration().expect("Failed to read configuration");
  configuration.database.database_name = Uuid::new_v4().to_string();

  let connection_pool = configure_database(&configuration.database).await;

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

  let server = run(listener, connection_pool.clone(), email_client)
    .expect("Failed to bind to address");
  let _ = spawn(server);

  TestApp {
    address,
    db_pool: connection_pool,
  }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
  // Create database
  let mut connection = PgConnection::connect_with(&config.without_db())
    .await
    .expect("Failed to connect to Postgres.");
  connection
    .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
    .await
    .expect("Failed to create database");

  // Migrate database
  let connection_pool = PgPool::connect_with(config.with_db())
    .await
    .expect("Failed to create database.");
  sqlx::migrate!("./migrations")
    .run(&connection_pool)
    .await
    .expect("Failed to migrate the database");
  connection_pool
}
