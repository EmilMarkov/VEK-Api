use crate::model::dto::auth::LoginRequest;
use crate::model::error::ErrorCode;
use crate::model::error::ErrorCode::{AUTH001, INTERNAL001};
use crate::prisma::{user, yandex_auth, PrismaClient};
use crate::service::user::UserService;
use actix_web::web;
use reqwest::Client;
use serde_json::Value;
use log::info;

pub async fn authenticate(
  login_request: LoginRequest,
  data: web::Data<PrismaClient>,
) -> Result<user::Data, ErrorCode> {
  let client = Client::new();
  let yandex_response = client
    .get(format!(
      "https://login.yandex.ru/info?oauth_token={}",
      login_request.access_token
    ))
    .send()
    .await;

  let yandex_response = match yandex_response {
    Ok(resp) => {
      if resp.status().is_success() {
        resp
      } else {
        return Err(AUTH001);
      }
    }
    Err(_) => {
      return Err(INTERNAL001);
    }
  };

  let yandex_info: Value = serde_json::from_str(&yandex_response.text().await.map_err(|_| INTERNAL001)?)
    .map_err(|_| INTERNAL001)?;

  let login = yandex_info["login"].as_str().ok_or(AUTH001)?;

  let user_data = if let Some(user) = UserService::get_user_by_login(&data, login).await.map_err(|_| INTERNAL001)? {
    UserService::update_user_timestamp(&data, &user.id).await.map_err(|_| INTERNAL001)?
  } else {
    UserService::create_user(&data, login).await.map_err(|_| INTERNAL001)?
  };

  if let Some(auth) = data
    .yandex_auth()
    .find_first(vec![yandex_auth::user_id::equals(Some(user_data.id.clone()))])
    .exec()
    .await
    .map_err(|_| INTERNAL001)? 
  {
    data.yandex_auth()
      .update(
        yandex_auth::access_token::equals(auth.access_token.clone()),
        vec![
            yandex_auth::token_type::set(login_request.token_type.clone()),
            yandex_auth::access_token::set(login_request.access_token.clone()),
            yandex_auth::expires_in::set(login_request.expires_in),
            yandex_auth::refresh_token::set(login_request.refresh_token.clone()),
            yandex_auth::scope::set(login_request.scope.clone()),
        ],
      )
      .exec()
      .await
      .map_err(|_| INTERNAL001)?;
  } else {
    data.yandex_auth()
      .create(
        login_request.token_type.clone(),
        login_request.access_token.clone(),
        login_request.expires_in,
        login_request.refresh_token.clone(),
        login_request.scope.clone(),
        vec![yandex_auth::user::connect(user::id::equals(user_data.id.clone()))],
      )
      .exec()
      .await
      .map_err(|_| INTERNAL001)?;
  }

  info!("Аутентификация завершена успешно для пользователя ID: {}", user_data.id);
  Ok(user_data)
}
