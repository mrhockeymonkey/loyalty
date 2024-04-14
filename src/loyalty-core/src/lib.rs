pub mod qr_gen;

use std::cmp::min;
use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};


#[async_trait]
pub trait StampCardTracker {
    async fn get_or_create_card(&mut self, card_id: &UserId) -> Result<BasicStampCard, String>;
    async fn stamp_card(&mut self, card_id: &UserId) -> Result<BasicStampCard, String>;
}

pub trait StampCard {
    fn add_stamp(&self); // result?
}
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserId(pub String);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicStampCard {
    user_id: UserId,
    pub stamps: u32, // TODO should not be public
    capacity: u32,
}

impl BasicStampCard {
    pub fn new(user_id: UserId) -> BasicStampCard {
        BasicStampCard {
            user_id,
            stamps: 0,
            capacity: 10
        }
    }

    pub fn with_stamp(&self) -> Self {
        BasicStampCard {
            user_id: self.user_id.clone(),
            stamps: min(self.stamps + 1, 10),
            capacity: self.capacity,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
