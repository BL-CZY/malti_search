use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;

use crate::structs::{Query, SearchEntry};

lazy_static! {
    pub static ref SEARCH_CONTEXT: Mutex<HashMap<(String, String), Vec<ObjectId>>> =
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
        .map(|str| SearchEntry::from_key(str))
        .collect::<Vec<SearchEntry>>();

    Ok(result)
}

pub fn append_context(keyword: &str, mode: &str, input: &[SearchEntry]) {
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

    let key = (keyword.to_string(), mode.to_string());

    if guard.contains_key(&key) {
        *guard.get_mut(&key).unwrap() = input.iter().map(|entry| entry.key.clone()).collect();

        return;
    }

    guard.insert(key, input.iter().map(|entry| entry.key.clone()).collect());
}
