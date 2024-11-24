use std::num::ParseIntError;

use regex::Regex;

pub struct Message {
    name: String,
    value: i32,
}

impl Message {
    pub fn new(name: &str, value: i32) -> Self {
        Self {
            name: String::from(name),
            value,
        }
    }

    pub fn decode(input: &str) -> Result<Self, ParseIntError> {
        let re = Regex::new(r"\[(.*?)\;(.*?)\]").unwrap();

        let mut name = String::new();
        let mut value = -1;
        for (_, [_name, _value]) in re.captures_iter(input).map(|c| c.extract()) {
            name = String::from(_name);
            value = _value.parse()?;
        }

        Ok(Self { name, value })
    }

    pub fn encode(&self) -> String {
        format!("[{};{}]", self.name, self.value)
    }
}
