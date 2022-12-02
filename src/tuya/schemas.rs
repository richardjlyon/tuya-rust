use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expire_time: u64,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokensResponse {
    pub result: Tokens,
    pub success: bool,
    pub t: u64,
    pub tid: String,
}
