pub fn trim_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    input.split_whitespace().for_each(|word| {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    });
    result
}
