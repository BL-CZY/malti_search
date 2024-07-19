use super::CLIENT;
use mongodb::{
    bson::{doc, Document},
    sync::{Collection, Database},
};
use serde_json;

#[napi]
pub fn get_word(word: String) -> String {
    println!("get word: {}", word);
    let db: Database = CLIENT.database("local");
    let col: Collection<Document> = db.collection("words");

    match col.find_one(doc! {"word": &word}, None) {
        Ok(res) => {
            if let Some(doc) = res {
                println!("found word {}", word);
                serde_json::to_string_pretty(&doc).unwrap()
            } else {
                println!("found word {}, but the value is corrupted", word);
                "".into()
            }
        }
        Err(_) => {
            println!("can't find word {}", word);
            "".into()
        }
    }
}
