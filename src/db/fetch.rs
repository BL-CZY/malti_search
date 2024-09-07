use std::str::FromStr;

use super::CLIENT;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use serde_json;

#[napi]
pub async fn get_word(word: String) -> String {
    #[cfg(feature = "log")]
    println!("getting word: {}", word);
    let db: Database = CLIENT.get().unwrap().database("local");
    let col: Collection<Document> = db.collection("words");

    let id = match ObjectId::from_str(&word) {
        Ok(res) => res,
        Err(_) => {
            #[cfg(feature = "log")]
            println!("Error parsing id");
            return "".into();
        }
    };

    if let Ok(Some(res)) = col.find_one(doc! {"_id": id}).await {
        #[cfg(feature = "log")]
        println!("found word {}", word);
        serde_json::to_string_pretty(&res).unwrap()
    } else {
        #[cfg(feature = "log")]
        println!("can't find it");
        "".into()
    }
}
