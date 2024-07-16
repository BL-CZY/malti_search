#[deny(clippy::all)]
pub mod db;
pub mod structs;
pub mod utils;

#[macro_use]
extern crate napi_derive;

#[cfg(test)]
mod tests {
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
}
