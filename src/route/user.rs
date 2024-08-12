use crate::prisma::PrismaClient;
use crate::service::user::UserService;
use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder};

pub fn user_controller_init(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/user")
      .service(get_current_user)
      .service(get_user_by_id),
  );
}

#[get("")]
async fn get_current_user(
  identity: Option<Identity>,
  client: web::Data<PrismaClient>,
) -> impl Responder {
  if let Some(identity) = identity {
    if let Ok(id) = identity.id() {
      match UserService::get_user_by_id(&client, &id).await {
        Ok(Some(user)) => {
          return HttpResponse::Ok().json(user);
        },
        Ok(None) => {
          log::warn!("Пользователь не найден в базе данных для id: {}", id);
          return HttpResponse::NotFound().body("User not found");
        },
        Err(err) => {
          log::error!("Ошибка при поиске пользователя: {:?}", err);
          return HttpResponse::InternalServerError().finish();
        }
      }
    }
  }
  log::warn!("Пользователь не авторизован");
  HttpResponse::Unauthorized().body("User not logged in")
}

#[get("/{id}")]
async fn get_user_by_id(
  path: web::Path<String>,
  client: web::Data<PrismaClient>,
) -> impl Responder {
  let id = path.into_inner();
  match UserService::get_user_by_id(&client, &id).await {
    Ok(Some(user)) => HttpResponse::Ok().json(user),
    Ok(None) => {
      log::warn!("Пользователь не найден в базе данных для id: {}", id);
      HttpResponse::NotFound().body("User not found")
    },
    Err(err) => {
      log::error!("Ошибка при поиске пользователя: {:?}", err);
      HttpResponse::InternalServerError().finish()
    },
  }
}
