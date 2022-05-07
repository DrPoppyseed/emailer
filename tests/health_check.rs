//! tests/health_check.rs

use emailer::{
  configuration::{get_configuration, DatabaseSettings},
  startup::run,
  telementry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use reqwest::Client;
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
async fn spawn_app() -> TestApp {
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

  let server =
    run(listener, connection_pool.clone()).expect("Failed to bind to address");
  let _ = spawn(server);

  TestApp {
    address,
    db_pool: connection_pool,
  }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
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

#[tokio::test]
async fn health_check_works() {
  let TestApp { address, .. } = spawn_app().await;
  let client = Client::new();

  let response = client
    .get(&format!("{}/health_check", &address))
    .send()
    .await
    .expect("Failed to execute request");

  assert!(response.status().is_success());
  assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
  let app = spawn_app().await;
  let client = Client::new();

  let body = "name=le%20guin&email=jau%40gmail.com";
  let response = client
    .post(&format!("{}/subscriptions", &app.address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(body)
    .send()
    .await
    .expect("Failed to execute request");

  assert_eq!(200, response.status().as_u16());

  let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscriptions");

  assert_eq!(saved.email, "jau@gmail.com");
  assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
  let TestApp {
    address: app_address,
    ..
  } = spawn_app().await;
  let client = Client::new();

  let test_cases = vec![
    ("name=le%20guin", "missing the email"),
    ("email=jua%40gmail.com", "missing the name"),
    ("", "missing both name and email"),
  ];

  for (invalid_body, error_message) in test_cases {
    let response = client
      .post(&format!("{}/subscriptions", &app_address))
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(invalid_body)
      .send()
      .await
      .expect("Failed to execute request");

    assert_eq!(
      400,
      response.status().as_u16(),
      "The API did not fail with status code 400 when payload was {}",
      error_message
    );
  }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
  let app = spawn_app().await;
  let client = reqwest::Client::new();
  let test_cases = vec![
    ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
    ("name=Ursula&email=", "empty email"),
    ("name=Ursula&email=definitely-not-an-email", "invalid email"),
  ];

  for (body, description) in test_cases {
    let response = client
      .post(&format!("{}/subscriptions", &app.address))
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(body)
      .send()
      .await
      .expect("Failed to execute request");

    assert_eq!(
      400,
      response.status().as_u16(),
      "The API did not return a 400 Bad Request when the payload was {}",
      description
    );
  }
}
