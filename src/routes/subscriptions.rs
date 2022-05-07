use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{
  web::{Data, Form},
  HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
  email: String,
  name: String,
}

impl TryFrom<FormData> for NewSubscriber {
  type Error = String;

  /// Converts wire-format data to valid domain model, or returns error string
  fn try_from(form: FormData) -> Result<Self, Self::Error> {
    let email = SubscriberEmail::parse(form.email)?;
    let name = SubscriberName::parse(form.name)?;
    Ok(Self { email, name })
  }
}

#[tracing::instrument(
  name = "Adding a new subscriber.",
  skip(form, pool),
  fields(
    subscriber_email = %form.email,
    subscriber_name = %form.name
  )
)]
pub async fn subscribe(
  form: Form<FormData>,
  pool: Data<PgPool>,
) -> HttpResponse {
  let new_subscriber = match form.0.try_into() {
    Ok(new) => new,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };
  match insert_subscriber(&pool, &new_subscriber).await {
    Ok(_) => HttpResponse::Ok().finish(),
    Err(_) => HttpResponse::InternalServerError().finish(),
  }
}

#[tracing::instrument(
  name = "Saving new subscriber details in the database",
  skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
  pool: &PgPool,
  new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
  sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
    Uuid::new_v4(),
    new_subscriber.email.as_ref(),
    new_subscriber.name.as_ref(),
    Utc::now()
  )
  .execute(pool)
  .await
  .map_err(|e| {
    tracing::error!("Failed to execute query: {:?}", e);
    e
    // By using the `?` operator, we can return the function early if it fails
    // with an sqlx::Error error.
  })?;
  Ok(())
}
