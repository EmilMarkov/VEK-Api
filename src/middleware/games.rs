use std::sync::Arc;
use tokio::sync::Mutex;

pub struct GamesMiddleware {
    api_key: Arc<Mutex<String>>,
}

impl GamesMiddleware {
    pub async fn new() -> Self {
        let api_key = Self::update_api_key().await.unwrap_or_else(|_| String::new());

        GamesMiddleware {
            api_key: Arc::new(Mutex::new(api_key)),
        }
    }

    pub async fn update_api_key() -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://rawg.io/";
        let resp = reqwest::get(url).await?;
        let body = resp.text().await?;

        let re = regex::Regex::new(r#""rawgApiKey":"([a-zA-Z0-9]+)""#)?;
        if let Some(caps) = re.captures(&body) {
            if let Some(api_key) = caps.get(1) {
                return Ok(api_key.as_str().to_string());
            }
        }

        Err("API Key not found".into())
    }

    pub async fn execute_with_retry<F, Fut, T>(&self, func: F) -> Result<T, reqwest::Error>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
    {
        let mut api_key = self.api_key.lock().await.clone();
        match func(api_key.clone()).await {
            Ok(result) => Ok(result),
            Err(err) => {
                if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    // Обновляем ключ API
                    if let Ok(new_api_key) = Self::update_api_key().await {
                        let mut key = self.api_key.lock().await;
                        *key = new_api_key.clone();
                        api_key = new_api_key.clone();
                    }
                }
                func(api_key).await
            }
        }
    }
}
