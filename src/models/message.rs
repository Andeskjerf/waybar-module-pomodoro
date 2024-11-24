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
                println!("{:?}", extracted);
                if extracted.1[0].is_empty() {
                    return Err(format!("message name is missing. msg == {:?}", extracted).into());
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let message = Message::new("example", 42);
        assert_eq!(message.name, "example");
        assert_eq!(message.value, 42);
    }

    #[test]
    fn test_name() {
        let message = Message::new("example", 42);
        assert_eq!(message.name(), "example");
    }

    #[test]
    fn test_value() {
        let message = Message::new("example", 42);
        assert_eq!(message.value(), 42);
    }

    #[test]
    fn test_encode() {
        let message = Message::new("example", 42);
        assert_eq!(message.encode(), "[example;42]");
    }

    #[test]
    fn test_decode_success() {
        let input = "[example;42]";
        let result = Message::decode(input);
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.name, "example");
        assert_eq!(message.value, 42);
    }

    #[test]
    fn test_decode_failure_missing_name() {
        let input = "[;7]";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(
                e.to_string(),
                "message name is missing. msg == (\"[;7]\", [\"\", \"7\"])"
            );
        }
    }

    #[test]
    fn test_decode_failure_no_input() {
        let input = "";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "unable to decode message: ");
        }
    }

    #[test]
    fn test_decode_failure_no_value() {
        let input = "[example;]";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "cannot parse integer from empty string");
        }
    }

    #[test]
    fn test_decode_failure_not_a_number() {
        let input = "[example;not_a_number]";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "invalid digit found in string");
        }
    }

    #[test]
    fn test_decode_missing_parts() {
        let input = "[example]";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "unable to decode message: [example]");
        }
    }

    #[test]
    fn test_decode_empty_input() {
        let input = "";
        let result = Message::decode(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "unable to decode message: ");
        }
    }
}
