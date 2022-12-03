use config::Config;
use std::{collections::HashMap, process, time::SystemTime};

// pub mod tuya;
pub mod error;

pub struct ApiSecrets {
    pub api_key: String,
    pub api_secret: String,
}

pub fn get_secrets() -> ApiSecrets {
    let api_key = get_key("api_key").unwrap_or_else(|err| {
        println!("Problem getting secrets: {err}");
        process::exit(1);
    });

    let api_secret = get_key("api_secret").unwrap_or_else(|err| {
        println!("Problem getting secrets: {err}");
        process::exit(1);
    });

    ApiSecrets {
        api_key,
        api_secret,
    }
}

pub fn get_sys_time() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let millis = n.as_millis();
            millis.to_string()
        }
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

// get value for key_name from secrets.toml
fn get_key(key_name: &str) -> Result<String, String> {
    let config = Config::builder()
        .add_source(config::File::with_name("secrets"))
        .build()
        .unwrap();

    let mut keys = config.try_deserialize::<HashMap<String, String>>().unwrap();

    match keys.remove(key_name) {
        Some(key) => Ok(key),
        None => Err(format!("No value found for key '{}'", key_name)),
    }
}
