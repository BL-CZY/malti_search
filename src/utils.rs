use std::cmp::min;

macro_rules! tail {
    ($str: ident) => {
        $str.chars().skip(1).collect::<String>().as_str()
    };
}

pub const LOG_ON: bool = false;

pub fn levdistance(a: &str, b: &str) -> u32 {
    // Levenshtein distance
    if a.is_empty() {
        b.len() as u32
    } else if b.is_empty() {
        a.len() as u32
    } else if a.chars().next().unwrap() == b.chars().next().unwrap() {
        levdistance(tail!(a), tail!(b))
    } else {
        1 + min(
            levdistance(tail!(a), b),
            min(levdistance(a, tail!(b)), levdistance(tail!(a), tail!(b))),
        )
    }
}

#[napi]
pub fn edit_distance(a: String, b: String) -> u32 {
    levdistance(&a, &b)
}
