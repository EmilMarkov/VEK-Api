use crate::middleware::games::GamesMiddleware;
use crate::modules::constants::API_SECRET;
use crate::prisma::PrismaClient;
use crate::route::auth::auth_controller_init;
use crate::route::games::games_controller_init;
use crate::route::health_check::health_check;
use crate::route::torrent::torrent_controller_init;
use crate::route::user::user_controller_init;
use crate::service::torrent::TorrentService;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_session::config::PersistentSession;
use actix_web::cookie::time::Duration;
use actix_web::dev::Server;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use log::{info, error};
use std::net::TcpListener;
use std::sync::Arc;

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
            error!("Ошибка при получении идентификации: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        },
    };

    info!("Запрос Index от пользователя: {}, количество сессий: {}", id, counter);
    Ok(HttpResponse::Ok().body(format!("Hello {id}, сессия: {counter}")))
}

pub fn get_config(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/api")
            .service(health_check)
            .configure(auth_controller_init)
            .configure(user_controller_init)
            .configure(games_controller_init)
            .configure(torrent_controller_init)
    );
}

pub async fn run(tcp_listener: TcpListener, data: PrismaClient) -> Result<Server, std::io::Error> {
    let data = Arc::new(data);  // Оборачиваем PrismaClient в Arc
    let data_web = web::Data::from(data.clone());  // Создаем web::Data<Arc<PrismaClient>>

    let games_middleware = GamesMiddleware::new().await;
    let games_middleware = web::Data::new(Arc::new(games_middleware));

    let private_key = actix_web::cookie::Key::from(API_SECRET.as_bytes());

    // Инициализация TorrentService и запуск процесса инициализации торрентов
    let torrent_service = TorrentService::new(data.clone());
    if let Err(err) = torrent_service.initialize_torrents().await {
        error!("Ошибка при инициализации торрентов: {:?}", err);
    }

    let torrent_service = web::Data::new(Arc::new(torrent_service));

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
            .app_data(data_web.clone())  // Передаем web::Data<Arc<PrismaClient>> в App
            .app_data(games_middleware.clone()) 
            .app_data(torrent_service.clone())
            .default_service(web::route().to(not_found))
            .service(index)
            .configure(get_config)
    })
    .listen(tcp_listener)
    .map_err(|e| {
        error!("Не удалось привязать адрес: {:?}", e);
        e
    })?
    .run();

    Ok(server)
}
