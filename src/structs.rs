use std::collections::HashSet;
use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::Serialize;

pub struct Query {
    pub keyword: String,
    pub skip: u32,
    pub limit: u32,
    pub max_dis: u32,
    pub found: HashSet<SearchEntry>,
    pub result: Vec<SearchEntry>,
    pub mode: String,
    pub mt: bool,
    pub en: bool,
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
            skip: 0,
            limit: 10,
            max_dis: 2,
            found: HashSet::new(),
            result: Vec::new(),
            mode: "".into(),
            mt: true,
            en: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult<'a> {
    pub key: String,
    pub word: &'a str,
    pub pos: &'a str,
    pub en: &'a Vec<String>,
    pub matched: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchEntry {
    pub key: ObjectId,
    pub word: String,
    pub pos: String,
    pub en: Vec<String>,
    pub matched: String,
}

impl SearchEntry {
    pub fn new() -> Self {
        SearchEntry {
            key: ObjectId::new(),
            word: "".into(),
            pos: "".into(),
            en: vec![],
            matched: "".into(),
        }
    }

    pub fn from_key(key: &ObjectId) -> Self {
        SearchEntry {
            key: key.to_owned(),
            ..Default::default()
        }
    }

    pub fn from_key_match(key: &ObjectId, matched: &str) -> Self {
        SearchEntry {
            key: key.to_owned(),
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
