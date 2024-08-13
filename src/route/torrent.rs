use actix_web::{get, web, HttpResponse, Responder};
use crate::model::dto::torrent::TorrentInfoRequest;
use crate::service::torrent::TorrentService;
use std::sync::Arc;

pub fn torrent_controller_init(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/torrent")
      .service(get_torrent_info),
  );
}

#[get("")]
async fn get_torrent_info(
  service: web::Data<Arc<TorrentService>>,
  data: web::Query<TorrentInfoRequest>,
) -> impl Responder {
  let game_name = &data.name;

  match service.search_torrent(game_name).await {
    Ok(results) => {
      if results.is_empty() {
        HttpResponse::NotFound().body("Torrent not found")
      } else {
        HttpResponse::Ok().json(results)
      }
    }
    Err(err) => HttpResponse::InternalServerError().body(err),
  }
}
