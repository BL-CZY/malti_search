use super::{
    context::{append_context, collect_context_words},
    CLIENT,
};
use crate::utils;
use crate::{db::context::SEARCH_CONTEXT, structs::SearchEntry};
use mongodb::{
    bson::{doc, Document},
    sync::{Collection, Database},
};
use serde_json::{self, json};
use std::collections::HashSet;

// returns true if the limit is 0
fn filter_res(
    search: &Vec<SearchEntry>,
    result: &mut Vec<SearchEntry>,
    found: &mut HashSet<SearchEntry>,
    skip_count: &mut u32,
    limit_count: &mut u32,
) {
    for res in search.iter() {
        if !found.contains(res) {
            if *skip_count != 0 {
                *skip_count -= 1;
                continue;
            } else {
                if *limit_count != 0 {
                    *limit_count -= 1;
                } else {
                    #[cfg(feature = "log")]
                    println!("limit {} reached", limit_count);
                    return;
                }
            }
            result.push(res.clone());
            found.insert(res.clone());
        }
    }
}

fn push_word(doc: &Document, matched: &str, distance: u32, result: &mut Vec<(SearchEntry, u32)>) {
    if let Ok(val) = doc.get_str("word") {
        result.push((SearchEntry::from_key_match(val, matched), distance));
    }
}

fn regular_search(
    keyword: &str,
    tokens_col: &Collection<Document>,
    max_dis: u32,
) -> Vec<SearchEntry> {
    let mut result: Vec<SearchEntry> = vec![];
    let mut temp_result: Vec<(SearchEntry, u32)> = vec![];
    let mut cursor;
    match tokens_col.find(None, None) {
        Ok(res) => cursor = res,
        Err(_) => {
            #[cfg(feature = "log")]
            println!("there is an error trying to find the keyword {}", keyword);
            return vec![];
        }
    }

    while let Some(res) = cursor.next() {
        if let Ok(doc) = res {
            if let Ok(val) = doc.get_str("exact") {
                let dis = utils::levdistance(val, keyword);
                if dis <= max_dis {
                    push_word(&doc, val, dis, &mut temp_result);
                }
            }
        }
    }

    temp_result.sort_by(|a, b| a.1.cmp(&b.1));

    for (str, dis) in temp_result.iter() {
        result.push(str.clone());
        #[cfg(feature = "log")]
        println!("found word {:?} with edit distance {}", str, dis);
    }

    result
}

fn handle_regular_search(
    keyword: &str,
    found: &mut HashSet<SearchEntry>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<SearchEntry>,
    col: &Collection<Document>,
    max_dis: u32,
) {
    let search_res: Vec<SearchEntry> = regular_search(keyword, col, max_dis);
    filter_res(&search_res, result, found, skip_count, limit_count);
}

// returns true if limit reaches 0
fn handle_keyword(
    keyword: &str,
    found: &mut HashSet<SearchEntry>,
    skip_count: &mut u32,
    limit_count: &mut u32,
    result: &mut Vec<SearchEntry>,
    db: &Database,
    max_dis: u32,
) {
    if let Ok(res) = collect_context_words(keyword, skip_count, limit_count) {
        *result = res;
        #[cfg(feature = "log")]
        println!("Found the result in the search context: {:?}", result);
        return;
    }

    // regular search
    handle_regular_search(
        keyword,
        found,
        skip_count,
        limit_count,
        result,
        &db.collection("tokens"),
        max_dis,
    );

    // update context
    append_context(keyword, result);

    #[cfg(feature = "log")]
    println!(
        "Updated the Search Context with word: \"{}\" to {:?}",
        keyword,
        SEARCH_CONTEXT.lock()
    );
}

pub fn fill_data(coll: &Collection<Document>, result: &mut SearchEntry) {
    let doc = if let Ok(Some(res)) = coll.find_one(
        doc! {
            "word": result.key.clone(),
        },
        None,
    ) {
        res
    } else {
        return;
    };

    result.word = if let Ok(val) = doc.get_str("surf") {
        val.into()
    } else {
        return;
    };

    result.pos = if let Ok(val) = doc.get_str("pos") {
        val.into()
    } else {
        return;
    };

    if let Ok(arr) = doc.get_array("en") {
        for ele in arr.iter() {
            if let Some(res) = ele.as_str() {
                result.en.push(res.into());
            }
        }
    }
}

#[napi]
pub fn search(query_str: String, skip: u32, limit: u32, max_dis: u32) -> String {
    let db = CLIENT.database("local");

    let keyword = query_str.replace("-", " ");
    let mut found: HashSet<SearchEntry> = HashSet::new();
    let mut result: Vec<SearchEntry> = vec![];
    let mut skip_count: u32 = skip;
    let mut limit_count: u32 = limit;

    #[cfg(feature = "log")]
    println!(
        "received query: query string \"{}\", skip: {}, limit: {}, max_distance {}",
        query_str, skip, limit, max_dis
    );

    handle_keyword(
        &keyword,
        &mut found,
        &mut skip_count,
        &mut limit_count,
        &mut result,
        &db,
        max_dis,
    );

    let word_coll: Collection<Document> = db.collection("words");

    for entry in result.iter_mut() {
        fill_data(&word_coll, entry);
    }

    #[cfg(feature = "log")]
    print!("result is: ");

    #[cfg(feature = "log")]
    for str in result.iter() {
        print!("{:?}, ", str);
    }

    #[cfg(feature = "log")]
    println!("");

    serde_json::to_string_pretty(&json!({
        "result": result
    }))
    .unwrap()
}
