use config::Config;
use reqwest::header::{self, HeaderValue};
use sha256::digest;
use std::collections::HashMap;
use std::str;
use std::time::SystemTime;

use hmac::{Hmac, Mac};
use sha2::Sha256;

#[tokio::main]
async fn main() {
    get_token().await;
}

async fn get_token() {
    get("token?grant_type=1").await;
}

async fn get(endpoint: &str) {
    // create signed payload
    let url_host = "openapi.tuyaeu.com";
    let ver = "v1.0";

    let api_key = get_key("api_key");
    let api_secret = get_key("api_secret");

    let payload = format!("{}{}", api_key, get_sys_time());
    let hash_body = digest("");

    let signed_payload = format!("{}GET\n{}\n\n/{}/{}", payload, hash_body, ver, endpoint);

    println!("Payload: {}", signed_payload);

    // sign payload
    let signature = hmac_signature(&api_secret, &signed_payload);

    println!("Secret: {}", api_secret);
    println!("payload: {}", signed_payload);
    println!("signature: {}", signature);

    // create headers
    let mut headers = header::HeaderMap::new();
    headers.insert("secret", HeaderValue::from_str(&api_secret).unwrap());
    headers.insert("client_id", HeaderValue::from_str(&api_key).unwrap());
    headers.insert("sign", HeaderValue::from_str(&signature).unwrap());
    headers.insert("t", HeaderValue::from_str(&get_sys_time()).unwrap());
    headers.insert("sign_method", HeaderValue::from_str("HMAC-SHA256").unwrap());

    println!("Headers: {:#?}", headers);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // get token
    let url = format!("https://{}/{}/{}", url_host, ver, endpoint);
    let response = client.get(url).send().await.unwrap();

    // println!("Status: {:#?}", response);
    let text = response.text().await.unwrap();
    println!("{}", text);
}

// get a key from secrets.toml
fn get_key(key_name: &str) -> String {
    let config = Config::builder()
        .add_source(config::File::with_name("secrets"))
        .build()
        .unwrap();

    let mut keys = config.try_deserialize::<HashMap<String, String>>().unwrap();

    match keys.remove(key_name) {
        Some(key) => key,
        None => {
            tracing::warn!("No API key found for key'{}'", key_name);
            std::process::exit(0);
        }
    }
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

// sign message with key using hmac
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
