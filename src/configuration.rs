//! src/configuration.rs
use std::env::{current_dir, var};

use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
  postgres::{PgConnectOptions, PgSslMode},
  ConnectOptions,
};

#[derive(Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub port: u16,
  pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: Secret<String>,
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub port: u16,
  pub host: String,
  pub database_name: String,
  pub require_ssl: bool,
}

/// Possible runtime environments for app
pub enum Environment {
  Local,
  Production,
}

impl Environment {
  pub fn as_str(&self) -> &'static str {
    match self {
      Environment::Local => "local",
      Environment::Production => "production",
    }
  }
}

impl TryFrom<String> for Environment {
  type Error = String;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    match s.to_lowercase().as_str() {
      "local" => Ok(Self::Local),
      "production" => Ok(Self::Production),
      other => Err(format!("{} is not a supported environment. Use either `local` or `production`.", other))
    }
  }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
  let base_path =
    current_dir().expect("Failed to determine the current directory");
  let configuration_directory = base_path.join("configuration");

  let environment: Environment = var("APP_ENV")
    .unwrap_or_else(|_| "local".into())
    .try_into()
    .expect("Failed to parse APP_ENV.");

  Config::builder()
    .add_source(
      File::from(configuration_directory.join("base.yml")).required(true),
    )
    .add_source(
      File::from(
        configuration_directory.join(environment.as_str().to_owned() + ".yml"),
      )
      .required(true),
    )
    .add_source(config::Environment::with_prefix("app").separator("__"))
    .build()?
    .try_deserialize()
}

impl DatabaseSettings {
  pub fn with_db(&self) -> PgConnectOptions {
    let mut options = self.without_db().database(&self.database_name);
    options.log_statements(tracing::log::LevelFilter::Trace);
    options
  }

  pub fn without_db(&self) -> PgConnectOptions {
    let ssl_mode = if self.require_ssl {
      PgSslMode::Require
    } else {
      PgSslMode::Prefer
    };
    PgConnectOptions::new()
      .host(&self.host)
      .username(&self.username)
      .password(&self.password.expose_secret())
      .port(self.port)
      .ssl_mode(ssl_mode)
  }
}
