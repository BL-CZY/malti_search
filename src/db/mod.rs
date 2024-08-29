pub mod context;
pub mod fetch;
pub mod search;

use lazy_static::lazy_static;
use mongodb::sync::Client;

const URI: &str = "mongodb://localhost:27017";

lazy_static! {
    static ref CLIENT: Client = Client::with_uri_str(URI).unwrap();
}
