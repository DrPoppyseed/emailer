use crate::routes::{health_check, subscribe};
use actix_web::{
  dev::Server,
  web::{get, post, Data},
  App, HttpServer,
};
use sqlx::PgPool;
use std::{io::Error, net::TcpListener};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, Error> {
  let connection = Data::new(db_pool);
  let server = HttpServer::new(move || {
    App::new()
      .route("/health_check", get().to(health_check))
      .route("/subscriptions", post().to(subscribe))
      .app_data(connection.clone())
  })
  .listen(listener)?
  .run();

  Ok(server)
}
