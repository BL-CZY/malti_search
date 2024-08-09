use super::CLIENT;
use mongodb::{
    bson::{doc, Document},
    sync::{Collection, Database},
};
use serde_json;

#[napi]
pub fn get_word(word: String) -> String {
    println!("getting word: {}", word);
    let db: Database = CLIENT.database("local");
    let col: Collection<Document> = db.collection("words");

    if let Ok(Some(res)) = col.find_one(doc! {"word": &word}, None) {
        println!("found word {}", word);
        serde_json::to_string_pretty(&res).unwrap()
    } else {
        println!("can't find it");
        "".into()
    }
}
