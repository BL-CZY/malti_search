use std::hash::Hash;

use serde::Serialize;

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
            word: "".into(),
            pos: "".into(),
            en: vec![],
            matched: "".into(),
        }
    }

    pub fn from_key_match(key: &str, matched: &str) -> Self {
        SearchEntry {
            key: key.into(),
            word: "".into(),
            pos: "".into(),
            en: vec![],
            matched: matched.into(),
        }
    }
}

impl PartialEq for SearchEntry {
    fn eq(&self, other: &Self) -> bool {
        return self.key == other.key;
    }
}

impl Eq for SearchEntry {}

impl Hash for SearchEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}
