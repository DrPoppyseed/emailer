use crate::{
  email_client::EmailClient,
  routes::{health_check, subscribe},
};
use actix_web::{
  dev::Server,
  web::{get, post, Data},
  App, HttpServer,
};
use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use tracing_actix_web::TracingLogger;

pub fn run(
  listener: TcpListener,
  db_pool: PgPool,
  email_client: EmailClient,
) -> Result<Server, Error> {
  let db_pool = Data::new(db_pool);

  let server = HttpServer::new(move || {
    App::new()
      .wrap(TracingLogger::default())
      .route("/health_check", get().to(health_check))
      .route("/subscriptions", post().to(subscribe))
      .app_data(db_pool.clone())
      .app_data(email_client.clone())
  })
  .listen(listener)?
  .run();

  Ok(server)
}
