use std::error::Error;

use regex::Regex;

#[derive(Debug, PartialEq)]
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn decode(input: &str) -> Result<Self, Box<dyn Error>> {
        let re = Regex::new(r"\[(.*?)\;(.*?)\]").unwrap();
        match re.captures(input) {
            Some(caps) => {
                let extracted: (&str, [&str; 2]) = caps.extract();
                Ok(Self {
                    name: extracted.1[0].to_string(),
                    value: extracted.1[1].parse()?,
                })
            }
            None => Err(format!("unable to decode message: {input}").into()),
        }
    }

    pub fn encode(&self) -> String {
        format!("[{};{}]", self.name, self.value)
    }
}
