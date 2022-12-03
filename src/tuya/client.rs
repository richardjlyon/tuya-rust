use super::schemas::{Tokens, TokensResponse};
use hmac::{Hmac, Mac};
use reqwest::{
    header::{self, HeaderValue},
    StatusCode,
};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use sha256::digest;
use std::{process, time::SystemTime};
use tuyascan::{error::AppError, get_key};

const APIBASE: &str = "openapi.tuyaeu.com";
const VER: &str = "v1.0";

// A client for implementing the TuYa API
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    // make an authenticated client
    pub fn new() -> Self {
        let client = build_client();
        Self { client }
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, AppError> {
        let url = format!("https://{}/{}/{}", APIBASE, VER, endpoint);
        let response = self
            .client
            .get(url)
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

    pub async fn get_tokens(&mut self) -> Result<Tokens, AppError> {
        self.get("token?grant_type=1")
            .await
            .map(|r: TokensResponse| r.result)
    }
}

fn build_client() -> reqwest::Client {
    let api_key = get_key("api_key").unwrap_or_else(|err| {
        println!("Problem getting secrets: {err}");
        process::exit(1);
    });
    
    let api_secret = get_key("api_secret").unwrap_or_else(|err| {
        println!("Problem getting secrets: {err}");
        process::exit(1);
    });
    
    let payload = dbg!(payload(&api_key));

    let signature = hmac_signature(&api_secret, &payload);

    let mut headers = header::HeaderMap::new();
    headers.insert("secret", HeaderValue::from_str(&api_secret).unwrap());
    headers.insert("client_id", HeaderValue::from_str(&api_key).unwrap());
    headers.insert("sign", HeaderValue::from_str(&signature).unwrap());
    headers.insert("t", HeaderValue::from_str(&get_sys_time()).unwrap());
    headers.insert("sign_method", HeaderValue::from_str("HMAC-SHA256").unwrap());

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

// construct the payload per the API
fn payload(api_key: &str) -> String {
    //FIXME refactor whole code to accept endpoint
    let endpoint = "token?grant_type=1";
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

fn get_sys_time() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let millis = n.as_millis();
            millis.to_string()
        }
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
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
