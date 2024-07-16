use std::cmp::min;

macro_rules! tail {
    ($str: ident) => {
        $str.chars().skip(1).collect::<String>().as_str()
    };
}

pub fn levdistance(a: &str, b: &str) -> u32 {
    // Levenshtein distance
    if a.is_empty() {
        return b.len() as u32;
    } else if b.is_empty() {
        return a.len() as u32;
    } else if a.chars().nth(0).unwrap() == b.chars().nth(0).unwrap() {
        return levdistance(tail!(a), tail!(b));
    } else {
        return 1 + min(
            levdistance(tail!(a), b),
            min(levdistance(a, tail!(b)), levdistance(tail!(a), tail!(b))),
        );
    }
}

#[napi]
pub fn edit_distance(a: String, b: String) -> u32 {
    return levdistance(&a, &b);
}
