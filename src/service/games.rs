use reqwest::Client;
use serde_json::Value;
use crate::model::dto::games::*;

const BASE_URL: &str = "https://api.rawg.io/api/";

pub struct GamesService;

impl GamesService {
    pub async fn get_game_list(api_key: String, request: GameListRequest) -> Result<Value, reqwest::Error> {
        let url = if let Some(next_url) = request.next {
            next_url
        } else {
            format!(
                "{}games?key={}&page={}&page_size=10&play_on_desktop=true",
                BASE_URL,
                api_key,
                request.page.unwrap_or(1)
            )
        };

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn search_game(api_key: String, request: GameSearchRequest) -> Result<Value, reqwest::Error> {
        let url = if let Some(next_url) = request.next {
            next_url
        } else {
            format!(
                "{}games?search={}&key={}",
                BASE_URL,
                request.query,
                api_key
            )
        };

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_game_details(api_key: String, request: GameDetailsRequest) -> Result<Value, reqwest::Error> {
        let url = format!("{}games/{}?key={}", BASE_URL, request.id, api_key);

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_game_screenshots(api_key: String, id: i32, request: GameScreenshotsRequest) -> Result<Value, reqwest::Error> {
        let url = format!(
            "{}games/{}/screenshots?key={}&page={}",
            BASE_URL,
            id,
            api_key,
            request.page.unwrap_or(1)
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_game_movies(api_key: String, id: i32, request: GameMoviesRequest) -> Result<Value, reqwest::Error> {
        let url = format!("{}games/{}/movies?key={}&page={}",
            BASE_URL,
            id,
            api_key,
            request.page.unwrap_or(1)
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }
}
