use lazy_static::lazy_static;
use mongodb::{
    bson::{doc, document, Document},
    sync::{Client, Collection},
};

const URI: &str = "mongodb://localhost:27017";

lazy_static! {
    static ref CLIENT: Client = Client::with_uri_str(URI).unwrap();
}

fn exact_search(
    keyword: &str,
    tokens_col: &Collection<Document>,
    words_col: &Collection<Document>,
) -> Vec<String> {
    let mut cursor;
    let mut result: Vec<String> = vec![];

    match tokens_col.find(
        doc! {
            "exact": keyword,
        },
        None,
    ) {
        Ok(cur) => {
            cursor = cur;
        }
        Err(_) => {
            return vec![];
        }
    }

    while let Some(res) = cursor.next() {
        if let Ok(document) = res {
            if let Some(val) = document.get("word") {
                result.push(val.to_string());
            }
        }
    }

    result
}

#[napi]
pub fn search(
    query_str: String,
    use_exact_index: bool,
    use_text_index: bool,
    skip: u32,
    limit: u32,
) {
    let db = CLIENT.database("local");
    println!("hello world!");

    let keywords: Vec<&str> = query_str.split(' ').collect::<Vec<&str>>();
}
