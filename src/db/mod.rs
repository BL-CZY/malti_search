pub mod context;
pub mod fetch;
pub mod search;

use mongodb::Client;
use once_cell::sync::OnceCell;

const URI: &str = "mongodb://localhost:27017";

static CLIENT: OnceCell<Client> = OnceCell::new();

#[napi]
pub async fn init() {
    CLIENT
        .set(Client::with_uri_str(URI).await.unwrap())
        .unwrap();
}
