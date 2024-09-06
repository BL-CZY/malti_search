use std::collections::HashSet;
use std::hash::Hash;

use serde::Serialize;

pub struct Query {
    pub keyword: String,
    pub mode: String,
    pub col_name: String,
    pub skip: u32,
    pub limit: u32,
    pub max_dis: u32,
    pub found: HashSet<SearchEntry>,
    pub result: Vec<SearchEntry>,
}

impl Default for Query {
    fn default() -> Self {
        Query::new()
    }
}

impl Query {
    pub fn new() -> Self {
        Query {
            keyword: "".into(),
            mode: "".into(),
            col_name: "".into(),
            skip: 0,
            limit: 10,
            max_dis: 2,
            found: HashSet::new(),
            result: Vec::new(),
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct SearchEntry {
    pub key: String,
    pub word: String,
    pub pos: String,
    pub en: Vec<String>,
    pub matched: String,
}

impl SearchEntry {
    pub fn new() -> Self {
        SearchEntry {
            key: "".into(),
            word: "".into(),
            pos: "".into(),
            en: vec![],
            matched: "".into(),
        }
    }

    pub fn from_key(key: &str) -> Self {
        SearchEntry {
            key: key.into(),
            ..Default::default()
        }
    }

    pub fn from_key_match(key: &str, matched: &str) -> Self {
        SearchEntry {
            key: key.into(),
            matched: matched.into(),
            ..Default::default()
        }
    }
}

impl Default for SearchEntry {
    fn default() -> Self {
        SearchEntry::new()
    }
}

impl PartialEq for SearchEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for SearchEntry {}

impl Hash for SearchEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}
