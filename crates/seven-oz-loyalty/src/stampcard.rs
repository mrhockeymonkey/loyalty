use std::cmp::min;

use actix_web::{HttpResponse, web};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use loyalty_core::UserId;

use crate::AppData;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicStampCard {
    user_id: UserId,
    pub stamps: u32, // TODO should not be public
    capacity: u32,
}

impl BasicStampCard {
    pub fn new(user_id: UserId) -> Self {
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


#[derive(Serialize)]
struct CardResponse {
    stamps: u32
}

// route handlers
pub async fn get_card(path: web::Path<String>, data: AppData) -> HttpResponse {
    let user_id = get_user_id(path);
    
    let mut tracker = data.cards.lock().unwrap();
    let card = tracker.get_or_create_card(&user_id).await.unwrap();
    
    let response = CardResponse { stamps: card.stamps };
    HttpResponse::Ok().json(response)
}

pub async fn reset_card(path: web::Path<String>, data: AppData) -> HttpResponse {
    let user_id = get_user_id(path);

    let mut tracker = data.cards.lock().unwrap();
    tracker.reset_card(&user_id).await.unwrap(); // TODO error handle
    
    
    HttpResponse::Ok().finish()
}

fn get_user_id(path: web::Path<String>) -> UserId {
    let user_id = path.into_inner();
    UserId(user_id)
}


