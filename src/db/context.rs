use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;

use crate::structs::SearchEntry;

lazy_static! {
    pub static ref SEARCH_CONTEXT: Mutex<HashMap<String, Vec<String>>> = Mutex::new(HashMap::new());
}

const CONTEXT_LIMIT: usize = 1024;

pub fn collect_context_words(
    keyword: &str,
    skip_count: &mut u32,
    limit_count: &mut u32,
) -> Result<Vec<SearchEntry>, ()> {
    let guard = match SEARCH_CONTEXT.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            #[cfg(feature = "log")]
            println!("Collect Context Words: The context is poisoned");
            poisoned.into_inner()
        }
    };

    if !guard.contains_key(keyword) {
        return Err(());
    }

    let result = guard
        .get(keyword)
        .unwrap()
        .iter()
        .skip(*skip_count as usize)
        .take(*limit_count as usize)
        .map(|str| SearchEntry::from_key(str))
        .collect::<Vec<SearchEntry>>();

    Ok(result)
}

pub fn append_context(keyword: &str, input: &Vec<SearchEntry>) {
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

    if guard.contains_key(keyword) {
        *guard.get_mut(keyword).unwrap() = input.iter().map(|entry| entry.key.clone()).collect();

        return;
    }

    guard.insert(
        keyword.to_string(),
        input.iter().map(|entry| entry.key.clone()).collect(),
    );
}
