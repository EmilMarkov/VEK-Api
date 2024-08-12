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
