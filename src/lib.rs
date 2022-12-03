use config::Config;
use std::collections::HashMap;

// pub mod tuya;
pub mod error;

// get value for key_name from secrets.toml
pub fn get_key(key_name: &str) -> Result<String, String> {
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


