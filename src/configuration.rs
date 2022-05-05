//! src/configuration.rs
use std::env::{current_dir, var};

use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
  pub port: u16,
  pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: Secret<String>,
  pub port: u16,
  pub host: String,
  pub database_name: String,
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
    .build()?
    .try_deserialize()
}

impl DatabaseSettings {
  pub fn connection_string(&self) -> Secret<String> {
    Secret::new(format!(
      "postgres://{}:{}@{}:{}/{}",
      self.username,
      self.password.expose_secret(),
      self.host,
      self.port,
      self.database_name
    ))
  }

  pub fn connection_string_without_db(&self) -> Secret<String> {
    Secret::new(format!(
      "postgres://{}:{}@{}:{}",
      self.username,
      self.password.expose_secret(),
      self.host,
      self.port
    ))
  }
}
