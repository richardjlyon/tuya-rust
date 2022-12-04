pub mod tuya;

use crate::tuya::client::Client;

#[tokio::main]
async fn main() {
    let client = Client::build().await;
}
