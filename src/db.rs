use lazy_static::lazy_static;
use mongodb::{
    bson::{doc, Document},
    sync::{Client, Collection, Cursor, Database},
};
use std::collections::HashSet;

const URI: &str = "mongodb://localhost:27017";

lazy_static! {
    static ref CLIENT: Client = Client::with_uri_str(URI).unwrap();
}

fn exact_search(keyword: &str, tokens_col: &Collection<Document>) -> Vec<String> {
    let mut cursor: Cursor<Document>;
    let mut result: Vec<String> = vec![];
    println!("searcing {}", keyword);

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
            println!("there is an error trying to find the keyword {}", keyword);
            return vec![];
        }
    }

    while let Some(res) = cursor.next() {
        if let Ok(document) = res {
            if let Some(val) = document.get("word") {
                result.push(val.to_string());
                println!("found {}", val.to_string());
            }
        }
    }

    result
}

// returns true if the limit is 0
fn handle_exact(
    exact_res: &Vec<String>,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<String>,
) -> bool {
    for res in exact_res.iter() {
        if !found.contains(res) {
            found.insert(res.clone());

            if *skip_count != 0 {
                *skip_count -= 1;
            } else {
                result.push(res.clone());
                *limit_count -= 1;
                if *limit_count == 0 {
                    return true;
                }
            }
        }
    }

    false
}

// returns true if limit reaches 0
fn handle_keyword(
    keyword: &str,
    exact_res: &mut Vec<String>,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<String>,
    use_exact_index: bool,
    db: &Database,
) -> bool {
    if use_exact_index {
        *exact_res = exact_search(keyword, &db.collection("tokens"));
        if handle_exact(&exact_res, found, skip_count, limit_count, result) {
            return true;
        }
    }

    false
}

#[napi]
pub fn search(
    query_str: String,
    use_exact_index: bool,
    use_text_index: bool,
    skip: u32,
    limit: u32,
) -> Vec<String> {
    let db = CLIENT.database("local");

    let keywords: Vec<&str> = query_str.split(' ').collect::<Vec<&str>>();
    let mut found: HashSet<String> = HashSet::new();
    let mut result: Vec<String> = vec![];
    let mut exact_res: Vec<String> = vec![];
    let mut skip_count: u32 = skip;
    let mut limit_count: u32 = limit;

    println!("received query: query string \"{}\", use exact_index: {}, use text_index: {}, skip: {}, limit: {}", query_str, use_exact_index, use_text_index, skip, limit);

    for keyword in keywords {
        if handle_keyword(
            keyword,
            &mut exact_res,
            &mut found,
            &mut skip_count,
            &mut limit_count,
            &mut result,
            use_exact_index,
            &db,
        ) {
            break;
        }
    }

    print!("result is: ");
    for str in result.iter() {
        print!("{}, ", str);
    }
    println!("");

    result
}
