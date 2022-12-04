use crate::{
    tuya::schemas::{Tokens, TokensResponse},
    Client,
};

use tuyascan::error::AppError;

impl Client {
    pub async fn auth(&mut self) {
        let tokens = self
            .get("token?grant_type=1")
            .await
            .map(|r: TokensResponse| r.result)
            .expect("Couldn't get token");

        self.tokens = Some(tokens);
    }
}
