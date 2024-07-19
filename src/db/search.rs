use super::CLIENT;
use crate::utils;
use mongodb::{
    bson::{doc, Document},
    sync::{Collection, Cursor, Database},
};
use serde_json::{self, json};
use std::collections::HashSet;

fn index_search(keyword: &str, tokens_col: &Collection<Document>, filter: Document) -> Vec<String> {
    let mut cursor: Cursor<Document>;
    let mut result: Vec<String> = vec![];
    println!("searching {}", keyword);

    match tokens_col.find(filter, None) {
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
                if let Some(v) = val.as_str() {
                    result.push(v.into());
                    println!("found {}", v);
                }
            }
        }
    }

    result
}

// returns true if the limit is 0
fn filter_res(
    search: &Vec<String>,
    result: &mut Vec<String>,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
) -> bool {
    for res in search.iter() {
        if !found.contains(res) {
            if *skip_count != 0 {
                *skip_count -= 1;
                continue;
            } else {
                if *limit_count != 0 {
                    *limit_count -= 1;
                } else {
                    return true;
                }
            }
            result.push(res.clone());
            found.insert(res.clone());
        }
    }

    false
}

// returns true if the limit is 0
fn handle_index_search(
    keyword: &str,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<String>,
    col: &Collection<Document>,
    filter: Document,
) -> bool {
    let search_res: Vec<String> = index_search(keyword, &col, filter);
    filter_res(&search_res, result, found, skip_count, limit_count)
}

fn regular_search(keyword: &str, tokens_col: &Collection<Document>, max_dis: u32) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    let mut temp_result: Vec<(String, u32)> = vec![];
    let mut cursor;
    match tokens_col.find(None, None) {
        Ok(res) => cursor = res,
        Err(_) => {
            println!("there is an error trying to find the keyword {}", keyword);
            return vec![];
        }
    }

    while let Some(res) = cursor.next() {
        if let Ok(doc) = res {
            if let Ok(val) = doc.get_str("exact") {
                let dis = utils::levdistance(val, keyword);
                if dis <= max_dis {
                    temp_result.push((val.into(), dis));
                }
            }
        }
    }

    temp_result.sort_by(|a, b| a.1.cmp(&b.1));

    for (str, dis) in temp_result.iter() {
        result.push(str.into());
        println!("found word {} with edit distance {}", str, dis);
    }

    result
}

fn handle_regular_search(
    keyword: &str,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<String>,
    col: &Collection<Document>,
    max_dis: u32,
) -> bool {
    let search_res: Vec<String> = regular_search(keyword, col, max_dis);
    filter_res(&search_res, result, found, skip_count, limit_count)
}

// returns true if limit reaches 0
fn handle_keyword(
    keyword: &str,
    found: &mut HashSet<String>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<String>,
    db: &Database,
    max_dis: u32,
) -> bool {
    if handle_index_search(
        keyword,
        found,
        skip_count,
        limit_count,
        result,
        &db.collection("tokens"),
        doc! { "exact": keyword },
    ) {
        return true;
    }

    // regular search
    if handle_regular_search(
        keyword,
        found,
        skip_count,
        limit_count,
        result,
        &db.collection("tokens"),
        max_dis,
    ) {
        return true;
    }

    false
}

#[napi]
pub fn search(query_str: String, skip: u32, limit: u32, max_dis: u32) -> String {
    let db = CLIENT.database("local");

    let keywords: Vec<&str> = query_str.split('-').collect::<Vec<&str>>();
    let mut found: HashSet<String> = HashSet::new();
    let mut result: Vec<String> = vec![];
    let mut skip_count: u32 = skip;
    let mut limit_count: u32 = limit;

    println!(
        "received query: query string \"{}\", skip: {}, limit: {}, max_distance {}",
        query_str, skip, limit, max_dis
    );

    for keyword in keywords {
        if handle_keyword(
            keyword,
            &mut found,
            &mut skip_count,
            &mut limit_count,
            &mut result,
            &db,
            max_dis,
        ) {
            break;
        }
    }

    print!("result is: ");
    for str in result.iter() {
        print!("{}, ", str);
    }
    println!("");

    serde_json::to_string_pretty(&json!({
        "result": result
    }))
    .unwrap()
}
