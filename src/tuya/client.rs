use crate::tuya::schemas::Tokens;
use reqwest::{
    header::{self, HeaderValue},
    StatusCode,
};
use serde::de::DeserializeOwned;
use sha256::digest;
use tuyascan::get_sys_time;
use tuyascan::{error::AppError, get_secrets, ApiSecrets};

use hmac::{Hmac, Mac};
use sha2::Sha256;

const APIBASE: &str = "openapi.tuyaeu.com";
const VER: &str = "v1.0";

// A client for implementing the TuYa API
#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    secrets: ApiSecrets,
    pub tokens: Option<Tokens>,
}

impl Client {
    // make an authenticated client
    pub async fn build() -> Client {
        let secrets = get_secrets();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "secret",
            HeaderValue::from_str(&secrets.api_secret).unwrap(),
        );
        headers.insert(
            "client_id",
            HeaderValue::from_str(&secrets.api_key).unwrap(),
        );
        headers.insert("sign_method", HeaderValue::from_str("HMAC-SHA256").unwrap());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let mut tuya_client = Self {
            client,
            secrets,
            tokens: None,
        };

        tuya_client.auth().await;
        println!("clietnt: {:#?}", tuya_client);

        tuya_client
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, AppError> {
        let url = format!("https://{}/{}/{}", APIBASE, VER, endpoint);
        let payload = payload(&self.secrets.api_key, endpoint);
        let signature = hmac_signature(&self.secrets.api_secret, &payload);

        let response = self
            .client
            .get(url)
            .header("sign", signature)
            .header("t", get_sys_time())
            .send()
            .await
            .map_err(|_| AppError::NetworkError)?;

        let status = response.status();

        let json_text = match status {
            StatusCode::OK => response.text().await.map_err(|_| AppError::ReadError),
            StatusCode::FORBIDDEN => Err(AppError::Authorisation),
            StatusCode::NOT_FOUND => Err(AppError::NotFound),
            _ => Err(AppError::Other),
        }?;

        println!("json: {}", json_text);

        let jd = &mut serde_json::Deserializer::from_str(&json_text);

        serde_path_to_error::deserialize(jd)
            .map_err(|e| AppError::ParseError(e.path().to_owned(), e.into_inner()))
    }
}

// impl Default for Client {
//     async fn default() -> Self {
//         Self::build().await
//     }
// }

// construct the payload per the API
fn payload(api_key: &str, endpoint: &str) -> String {
    let hash_body = digest("");
    format!(
        "{}{}GET\n{}\n\n/{}/{}",
        api_key,
        get_sys_time(),
        hash_body,
        VER,
        endpoint
    )
}

// compute the HMAC signature per the API
// see: https://developer.tuya.com/en/docs/iot/singnature?id=Ka43a5mtx1gsc
fn hmac_signature(key: &str, msg: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    mac.update(msg.as_bytes());

    let code_bytes = mac.finalize().into_bytes();

    let hex_bytes = hex::encode(&code_bytes);

    hex_bytes.to_uppercase()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_signs_correctly() {
        let key = "ab99383d99fb4c09aec25572277875e8";
        let msg = "qn4ry993w4syyfv8etfs1669887958704GET\ne3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\n/v1.0/token?grant_type=1";
        let expected = "15461121A1B47A7BBF86F1F159BFD1D545EF2B05F97597F21BCA656E2C2A0E1F";

        let signature = hmac_signature(key, msg);

        assert_eq!(signature, expected);
    }
}
