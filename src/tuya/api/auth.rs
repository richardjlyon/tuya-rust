use crate::{
    tuya::schemas::{Tokens, TokensResponse},
    Client,
};

use tuyascan::error::AppError;

impl Client {
    pub async fn auth(&self) -> Result<Tokens, AppError> {
        self.get("token?grant_type=1")
            .await
            .map(|r: TokensResponse| r.result)
    }
}
