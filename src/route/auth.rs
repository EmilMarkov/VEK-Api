use crate::model::dto::auth::LoginRequest;
use crate::model::error::ErrorResponse;
use crate::prisma::PrismaClient;
use crate::service::auth::authenticate;

use actix_identity::Identity;
use actix_web::web::Json;
use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

#[allow(dead_code)]
pub fn auth_controller_init(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/auth")
      .service(login)
      .service(logout)
  );
}

#[post("")]
async fn login(
  body: Json<LoginRequest>,
  req: HttpRequest,
  data: web::Data<PrismaClient>,
) -> impl Responder {
  let auth_result = authenticate(body.into_inner(), data).await;
  match auth_result {
    Ok(user_data) => {
      Identity::login(&req.extensions_mut(), user_data.id.clone()).unwrap();
      HttpResponse::Ok().json(user_data)
    }
    Err(e) => ErrorResponse::build(e),
  }
}

#[post("/logout")]
async fn logout(ident: Identity) -> impl Responder {
  ident.logout();
  HttpResponse::Ok().finish()
}
