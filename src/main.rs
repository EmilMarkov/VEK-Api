use std::{env, net::TcpListener};

use actix_web::Result;
use modules::helpers::set_database_url;

use prisma::PrismaClient;

use crate::server::run;

mod middleware;
mod model;
#[allow(warnings, unused)]
mod prisma;
mod route;
mod server;
mod service;
mod modules;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  let _ = set_database_url();

  let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
    String::from("file:veklauncher.db")
  });

  let data = PrismaClient::_builder().with_url(database_url).build().await.unwrap();

  if let Err(e) = data._migrate_deploy().await {
    eprintln!("Failed to run database migrations: {:?}", e);
    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to run database migrations"));
  }

  env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
  env::set_var("RUST_BACKTRACE", "1");
  env::set_var("RUST_LOG", "actix_web=debug");

  let listener = TcpListener::bind("127.0.0.1:8004").expect("Failed to bind address");

  run(listener, data).await?.await?;

  Ok(())
}
