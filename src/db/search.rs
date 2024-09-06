use super::{
    context::{append_context, collect_context_words},
    CLIENT,
};

use crate::{db::context::SEARCH_CONTEXT, structs::SearchEntry};
use crate::{structs::Query, utils};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Cursor, {Collection, Database},
};
use serde_json::{self, json};

// returns true if the limit is 0
fn filter_res(search: &[SearchEntry], query: &mut Query) {
    for res in search.iter() {
        if !query.found.contains(res) {
            if query.skip != 0 {
                query.skip -= 1;
                continue;
            } else if query.limit != 0 {
                query.limit -= 1;
            } else {
                #[cfg(feature = "log")]
                println!("limit {} reached", query.limit);
                return;
            }

            query.result.push(res.clone());
            query.found.insert(res.clone());
        }
    }
}

fn push_word(doc: &Document, matched: &str, distance: u32, result: &mut Vec<(SearchEntry, u32)>) {
    if let Ok(val) = doc.get_str("word") {
        result.push((SearchEntry::from_key_match(val, matched), distance));
    }
}

async fn regular_search(
    keyword: &str,
    tokens_col: &Collection<Document>,
    max_dis: u32,
) -> Vec<SearchEntry> {
    let mut result: Vec<SearchEntry> = vec![];
    let mut temp_result: Vec<(SearchEntry, u32)> = vec![];
    let mut cursor: Cursor<Document>;
    match tokens_col.find(None, None).await {
        Ok(res) => cursor = res,
        Err(_) => {
            #[cfg(feature = "log")]
            println!("there is an error trying to find the keyword {}", keyword);
            return vec![];
        }
    }

    while let Ok(Some(doc)) = cursor.try_next().await {
        if let Ok(val) = doc.get_str("exact") {
            let dis = utils::levdistance(val, keyword);
            if dis <= max_dis {
                push_word(&doc, val, dis, &mut temp_result);
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

async fn handle_regular_search(query: &mut Query, col: &Collection<Document>) {
    let search_res: Vec<SearchEntry> = regular_search(&query.keyword, col, query.max_dis).await;
    filter_res(&search_res, query);
}

// returns true if limit reaches 0
async fn handle_keyword(query: &mut Query, db: &Database) {
    if let Ok(res) = collect_context_words(query) {
        query.result = res;
        #[cfg(feature = "log")]
        println!("Found the result in the search context: {:?}", query.result);
        return;
    }

    // regular search
    handle_regular_search(query, &db.collection(&query.col_name)).await;

    // update context
    append_context(&query.keyword, &query.mode, &query.result);

    #[cfg(feature = "log")]
    println!(
        "Updated the Search Context with word: \"{}\" to {:?}",
        query.keyword,
        SEARCH_CONTEXT.lock()
    );
}

async fn fill_data(coll: &Collection<Document>, result: &mut SearchEntry) {
    let doc = if let Ok(Some(res)) = coll
        .find_one(
            doc! {
                "word": result.key.clone(),
            },
            None,
        )
        .await
    {
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
pub async fn search(
    query_str: String,
    skip: u32,
    limit: u32,
    max_dis: u32,
    mode: String,
) -> String {
    let db = CLIENT.get().unwrap().database("local");

    let mut query = Query {
        keyword: query_str.clone().replace("-", " "),
        col_name: match mode.as_str() {
            "m" => "mt-tokens".to_string(),
            "e" => "en-tokens".to_string(),
            "b" => "tokens".to_string(),
            _ => "tokens".to_string(),
        },
        skip,
        limit,
        max_dis,
        mode: mode.clone(),
        ..Default::default()
    };

    #[cfg(feature = "log")]
    println!(
        "received query: query string \"{}\", skip: {}, limit: {}, max_distance {}",
        query.keyword, query.skip, query.limit, query.max_dis
    );

    handle_keyword(&mut query, &db).await;

    let word_coll: Collection<Document> = db.collection("words");

    for entry in query.result.iter_mut() {
        fill_data(&word_coll, entry).await;
    }

    #[cfg(feature = "log")]
    print!("result is: ");

    #[cfg(feature = "log")]
    for str in query.result.iter() {
        print!("{:?}, ", str);
    }

    #[cfg(feature = "log")]
    println!();

    serde_json::to_string_pretty(&json!({
        "result": query.result
    }))
    .unwrap()
}
