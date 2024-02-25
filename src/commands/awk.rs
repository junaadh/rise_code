pub fn awk(str: &str, no: i32) -> String {
    let no = no as usize;
    let str_slice: Vec<&str> = str.split_whitespace().collect();
    str_slice.get(no - 1).unwrap_or(&" ").to_string()
}
