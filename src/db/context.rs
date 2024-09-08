use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;

use crate::structs::{Query, SearchEntry};

lazy_static! {
    pub static ref SEARCH_CONTEXT: Mutex<HashMap<(String, String), Vec<(ObjectId, String)>>> =
        Mutex::new(HashMap::new());
}

const CONTEXT_LIMIT: usize = 1024;

pub fn collect_context_words(query: &Query) -> Result<Vec<SearchEntry>, String> {
    let guard = match SEARCH_CONTEXT.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            #[cfg(feature = "log")]
            println!("Collect Context Words: The context is poisoned");
            poisoned.into_inner()
        }
    };

    let key = (query.keyword.clone(), query.mode.clone());

    if !guard.contains_key(&key) {
        return Err("Already".into());
    }

    let result = guard
        .get(&key)
        .unwrap()
        .iter()
        .skip(query.skip as usize)
        .take(query.limit as usize)
        .map(|(id, matched)| SearchEntry::from_key_match(id, matched))
        .collect::<Vec<SearchEntry>>();

    Ok(result)
}

pub fn append_context(query: &Query, result: &[SearchEntry]) {
    let mut guard = match SEARCH_CONTEXT.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            #[cfg(feature = "log")]
            println!("Update Context: The context is poisoned");
            poisoned.into_inner()
        }
    };

    if guard.len() > CONTEXT_LIMIT {
        return;
    }

    let key = (query.keyword.to_string(), query.mode.to_string());

    if guard.contains_key(&key) {
        *guard.get_mut(&key).unwrap() = result
            .iter()
            .map(|entry| (entry.key.clone(), entry.matched.clone()))
            .collect();

        return;
    }

    guard.insert(
        key,
        result
            .iter()
            .map(|entry| (entry.key.clone(), entry.matched.clone()))
            .collect(),
    );
}
