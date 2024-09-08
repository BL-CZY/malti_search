use super::{
    context::{append_context, collect_context_words},
    CLIENT, EN_TOKENS, MT_TOKENS,
};
use rayon::prelude::*;

use crate::structs::{Query, SearchResult};
use crate::{db::context::SEARCH_CONTEXT, structs::SearchEntry};
use mongodb::{
    bson::{doc, Document},
    Collection,
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

async fn regular_search(query: &mut Query) -> Vec<SearchEntry> {
    let mut result: Vec<SearchEntry> = vec![];
    let mut temp_result: Vec<(SearchEntry, u32)> = vec![];

    if query.mt {
        temp_result.extend(
            MT_TOKENS
                .get()
                .unwrap()
                .par_iter()
                .filter_map(|(val, id)| {
                    let dis = levenshtein::levenshtein(val, &query.keyword) as u32;
                    if dis <= query.max_dis {
                        Some((SearchEntry::from_key_match(&id, &query.keyword), dis))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(SearchEntry, u32)>>(),
        );
    }

    if query.en {
        temp_result.extend(
            EN_TOKENS
                .get()
                .unwrap()
                .par_iter()
                .filter_map(|(val, id)| {
                    let dis = levenshtein::levenshtein(val, &query.keyword) as u32;
                    if dis <= query.max_dis {
                        Some((SearchEntry::from_key_match(&id, &query.keyword), dis))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(SearchEntry, u32)>>(),
        );
    }

    temp_result.par_sort_by(|a, b| a.1.cmp(&b.1));

    for (str, dis) in temp_result.iter() {
        result.push(str.clone());
        #[cfg(feature = "log")]
        println!("found word {:?} with edit distance {}", str, dis);
    }

    result
}

async fn handle_regular_search(query: &mut Query) {
    let search_res: Vec<SearchEntry> = regular_search(query).await;
    append_context(&query, &search_res);

    filter_res(&search_res, query);
}

// returns true if limit reaches 0
async fn handle_keyword(query: &mut Query) {
    if let Ok(res) = collect_context_words(query) {
        query.result = res;
        #[cfg(feature = "log")]
        println!("Found the result in the search context: {:?}", query.result);
        return;
    }

    // regular search
    handle_regular_search(query).await;

    #[cfg(feature = "log")]
    println!(
        "Updated the Search Context with word: \"{}\" to {:?}",
        query.keyword,
        SEARCH_CONTEXT.lock()
    );
}

async fn fill_data(coll: &Collection<Document>, result: &mut SearchEntry) {
    let doc = if let Ok(Some(res)) = coll
        .find_one(doc! {
            "_id": result.key.clone(),
        })
        .await
    {
        res
    } else {
        return;
    };

    #[cfg(feature = "log")]
    println!("found document while filling data: {}", doc);

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
    if query_str.is_empty() {
        return "".into();
    }

    let db = CLIENT.get().unwrap().database("local");
    let (mt, en): (bool, bool) = match mode.as_str() {
        "m" => (true, false),
        "e" => (false, true),
        "b" => (true, true),
        _ => (true, true),
    };

    let mut query = Query {
        keyword: query_str.clone().replace("-", " "),
        skip,
        limit,
        max_dis,
        mt,
        en,
        mode,
        ..Default::default()
    };

    #[cfg(feature = "log")]
    println!(
        "received query: query string \"{}\", skip: {}, limit: {}, max_distance {}",
        query.keyword, query.skip, query.limit, query.max_dis
    );

    handle_keyword(&mut query).await;

    let word_col: Collection<Document> = db.collection("words");
    for entry in query.result.iter_mut() {
        fill_data(&word_col, entry).await;
    }

    #[cfg(feature = "log")]
    print!("result is: ");

    #[cfg(feature = "log")]
    for str in query.result.iter() {
        print!("{:?}, ", str);
    }

    #[cfg(feature = "log")]
    println!();

    let result = query
        .result
        .par_iter()
        .map(move |v| SearchResult {
            key: v.key.to_string(),
            word: &v.word,
            pos: &v.pos,
            en: &v.en,
            matched: &v.matched,
        })
        .collect::<Vec<SearchResult>>();

    serde_json::to_string_pretty(&json!({
        "result": result,
    }))
    .unwrap()
}
