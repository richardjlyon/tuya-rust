pub mod tuya;

use crate::tuya::client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new();
    let tokens = client.get_tokens().await;

    println!("{:#?}", tokens);
}
