pub fn grep(str: &str, search_term: &str) -> i32 {
    if let Some(index) = str.split_whitespace().position(|word| word == search_term) {
        (index + 1) as i32
    } else {
        -1
    }
}
