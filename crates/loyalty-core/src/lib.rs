use std::cmp::min;
use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod qr_gen;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserId(pub String);

pub struct PhoneNumber {
    number: [u32;11]
}

impl TryFrom<&str> for PhoneNumber {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let num: Vec<u32> = value.chars()
            .map(|c| c.to_digit(10))
            .filter_map(|x| x)
            .collect();
        
        match num.as_slice() {
            num if num.len() != 11 => Err("Number must be 11 digits"),
            [first, ..] if first != &0 => Err("First digit must be 0"),
            [_, second, ..] if second != &7 => Err("Second digit must be 7"),
            num => { 
                let mut number = [0u32;11];
                number.copy_from_slice(num);
                Ok(PhoneNumber { number }) 
            }
        }
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

