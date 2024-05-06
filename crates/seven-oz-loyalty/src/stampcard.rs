use actix_web::{HttpResponse, web};
use log::info;
use mongodb::bson::doc;
use mongodb::Collection;
use serde::Serialize;
use thiserror::Error;
use loyalty_core::{BasicStampCard, UserId};
use crate::{AppData};

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


