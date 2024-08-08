#[deny(clippy::all)]
pub mod db;
pub mod structs;
pub mod utils;

#[macro_use]
extern crate napi_derive;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::structs::SearchEntry;

    use super::utils::levdistance;

    #[test]
    fn levdistance_test() {
        assert_eq!(levdistance("book", "back"), 2);
        assert_eq!(levdistance("back", "book"), 2);
        assert_eq!(levdistance("book", "backend"), 5);
        assert_eq!(levdistance("backend", "book"), 5);
        assert_eq!(levdistance("għandna", "gand"), 3);
        assert_eq!(levdistance("ġaa", "ġġa"), 1);
    }

    #[test]
    fn search_entry_test() {
        let mut s: HashSet<SearchEntry> = HashSet::new();
        s.insert(SearchEntry::from_key("a"));
        let mut test = SearchEntry::from_key("a");
        test.pos = "b".into();
        s.insert(test);

        let mut s2: HashSet<SearchEntry> = HashSet::new();
        s2.insert(SearchEntry::from_key("a"));

        assert_eq!(s, s2);
    }
}
