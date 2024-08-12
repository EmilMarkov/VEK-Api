use crate::modules::constants::API_SECRET;
use crate::prisma::PrismaClient;
use crate::route::auth::auth_controller_init;
use crate::route::health_check::health_check;
use crate::route::user::user_controller_init;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_session::config::PersistentSession;
use actix_web::cookie::time::Duration;
use actix_web::dev::Server;
use actix_web::web::{scope, ServiceConfig};
use actix_web::{error, get, middleware, web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;
use log::{info, error};

pub async fn not_found() -> actix_web::Result<impl Responder> {
  Ok(HttpResponse::NotFound().body("Not Found"))
}

#[get("/")]
async fn index(identity: Option<Identity>, session: Session) -> actix_web::Result<impl Responder> {
  let counter: i32 = session
    .get::<i32>("counter")
    .unwrap_or(Some(0))
    .unwrap_or(0);

  let id = match identity.map(|id| id.id()) {
    None => "anonymous".to_owned(),
    Some(Ok(id)) => id,
    Some(Err(err)) => {
      error!("Error retrieving identity: {:?}", err);
      return Err(error::ErrorInternalServerError(err));
    },
  };

  info!("Index request from user: {}, session count: {}", id, counter);
  Ok(HttpResponse::Ok().body(format!("Hello {id}, session: {counter}")))
}

#[doc = "Setup the service served by the application."]
pub fn get_config(conf: &mut ServiceConfig) {
  conf.service(
    scope("/api")
      .service(health_check)
      .configure(auth_controller_init)
      .configure(user_controller_init),
  );
}

#[doc = "Create the server instance."]
pub async fn run(tcp_listener: TcpListener, data: PrismaClient) -> Result<Server, std::io::Error> {
  let data = web::Data::new(data);

  let private_key = actix_web::cookie::Key::from(API_SECRET.as_bytes());

  let server = HttpServer::new(move || {
    App::new()
      .wrap(IdentityMiddleware::default())
      .wrap(
        SessionMiddleware::builder(CookieSessionStore::default(), private_key.clone())
          .cookie_name("app_session".to_owned())
          .session_lifecycle(PersistentSession::default().session_ttl(Duration::hours(24)))
          .cookie_secure(false)
          .build(),
      )
      .wrap(middleware::NormalizePath::trim())
      .wrap(middleware::Logger::default())
      .app_data(data.clone())
      .default_service(web::route().to(not_found))
      .service(index)
      .configure(get_config)
  })
  .listen(tcp_listener)
  .map_err(|e| {
    error!("Failed to bind address: {:?}", e);
    e
  })?
  .run();

  Ok(server)
}
