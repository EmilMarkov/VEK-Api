use actix_identity::Identity;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use std::future::ready;
use log::{error, warn};
use reqwest::Client;
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Deserialize)]
struct YandexTokenInfo {
  pub id: String,
  pub login: String,
  pub client_id: String,
  pub psuid: String,
}

impl FromRequest for YandexTokenInfo {
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

  fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
    let identity_result = Identity::from_request(req, &mut Payload::None).into_inner();

    if let Ok(identity) = identity_result {
      if let Some(access_token) = identity.id().ok() {
        let client = Client::new();
        let request_url = format!("https://login.yandex.ru/info?oauth_token={}", access_token);

        return Box::pin(async move {
          match client.get(&request_url).send().await {
            Ok(resp) => {
              if resp.status().is_success() {
                match resp.json::<YandexTokenInfo>().await {
                  Ok(token_info) => {
                    Ok(YandexTokenInfo {
                      id: token_info.id,
                      login: token_info.login,
                      client_id: token_info.client_id,
                      psuid: token_info.psuid,
                    })
                  }
                  Err(e) => {
                    error!("Не удалось десериализовать ответ от Yandex API: {:?}", e);
                    Err(actix_web::error::ErrorInternalServerError("Не удалось разобрать ответ от Yandex API"))
                  }
                }
              } else {
                warn!("Недействительный ответ токена от Yandex API. Статус: {}", resp.status());
                Err(actix_web::error::ErrorUnauthorized("Недействительный ответ от Yandex API"))
              }
            }
            Err(e) => {
              error!("Не удалось подключиться к Yandex API: {:?}", e);
              Err(actix_web::error::ErrorInternalServerError("Не удалось подключиться к Yandex API"))
            }
          }
        });
      } else {
        warn!("Найдена идентификация, но токен доступа отсутствует.");
      }
    } else {
      warn!("Не удалось получить идентификацию.");
    }

    warn!("Идентификация не найдена, несанкционированный доступ.");
    Box::pin(ready(Err(actix_web::error::ErrorUnauthorized("Несанкционированный доступ"))))
  }
}
