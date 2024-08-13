pub mod auth {
  use serde::{Deserialize, Serialize};

  #[doc = "User Login"]
  #[derive(Serialize, Debug, Deserialize)]
  pub struct LoginRequest {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
    pub scope: String,
  }
}

pub mod games {
  use serde::{Deserialize, Serialize};

  #[derive(Clone)] 
  #[derive(Serialize, Deserialize)]
  pub struct GameListRequest {
    pub page: Option<usize>,
    pub next: Option<String>,
  }

  #[derive(Clone)] 
  #[derive(Serialize, Deserialize)]
  pub struct GameSearchRequest {
    pub query: String,
    pub next: Option<String>,
  }

  #[derive(Clone)] 
  #[derive(Serialize, Deserialize)]
  pub struct GameDetailsRequest {
    pub id: i32,
  }

  #[derive(Clone)] 
  #[derive(Serialize, Deserialize)]
  pub struct GameScreenshotsRequest {
    pub page: Option<usize>,
    pub next: Option<String>,
  }

  #[derive(Clone)] 
  #[derive(Serialize, Deserialize)]
  pub struct GameMoviesRequest {
    pub page: Option<usize>,
    pub next: Option<String>,
  }
}
