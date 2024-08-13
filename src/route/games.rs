use actix_web::{get, post, web, HttpResponse, Responder};
use crate::model::dto::games::*;
use crate::service::games::GamesService;
use crate::middleware::games::GamesMiddleware;
use std::sync::Arc;

#[allow(dead_code)]
pub fn games_controller_init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/games")
            .service(get_game_list)
            .service(search_game)
            .service(get_game_details)
            .service(get_game_screenshots)
            .service(get_game_movies)
    );
}

#[get("")]
async fn get_game_list(
    middleware: web::Data<Arc<GamesMiddleware>>,
    data: web::Query<GameListRequest>
) -> impl Responder {
    let request = data.into_inner();
    match middleware
        .execute_with_retry(move |api_key| {
            let req = request.clone(); // Клонирование переменной request
            async move { GamesService::get_game_list(api_key, req).await }
        })
        .await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/search")]
async fn search_game(
    middleware: web::Data<Arc<GamesMiddleware>>,
    data: web::Json<GameSearchRequest>
) -> impl Responder {
    let request = data.into_inner();
    match middleware
        .execute_with_retry(move |api_key| {
            let req = request.clone(); // Клонирование переменной request
            async move { GamesService::search_game(api_key, req).await }
        })
        .await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{id}")]
async fn get_game_details(
    middleware: web::Data<Arc<GamesMiddleware>>,
    path: web::Path<i32>,
) -> impl Responder {
    let request_id = path.into_inner();
    match middleware
        .execute_with_retry(move |api_key| {
            async move { GamesService::get_game_details(api_key, GameDetailsRequest { id: request_id }).await }
        })
        .await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{id}/screenshots")]
async fn get_game_screenshots(
    middleware: web::Data<Arc<GamesMiddleware>>,
    path: web::Path<i32>,
    data: web::Query<GameScreenshotsRequest>
) -> impl Responder {
    let request_id = path.into_inner();
    let request = data.into_inner();
    match middleware
        .execute_with_retry(move |api_key| {
            let req = GameScreenshotsRequest {
                page: request.page,
                next: request.next.clone(),
            };
            async move { GamesService::get_game_screenshots(api_key, request_id, req).await }
        })
        .await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{id}/movies")]
async fn get_game_movies(
    middleware: web::Data<Arc<GamesMiddleware>>,
    path: web::Path<i32>,
    data: web::Query<GameMoviesRequest>
) -> impl Responder {
    let request_id = path.into_inner();
    let request = data.into_inner();
    match middleware
        .execute_with_retry(move |api_key| {
            let req = GameMoviesRequest {
                page: request.page,
                next: request.next.clone(),
            };
            async move { GamesService::get_game_movies(api_key, request_id, req).await }
        })
        .await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
