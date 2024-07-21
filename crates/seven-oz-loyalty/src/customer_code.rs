use actix_web::dev::ConnectionInfo;
use actix_web::{get, HttpResponse, post, web};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use loyalty_core::qr_gen::CustomerQrCode;
use loyalty_core::UserId;
use crate::AppData;

#[derive(Serialize)]
struct CodeResponse {
    code: String
}

#[derive(Deserialize, Serialize)]
pub struct ClaimRequest {
    id: String,
    code: String
}

pub async fn get_code(data: AppData, conn: ConnectionInfo) -> HttpResponse {
    info!("getting QR");
    
    let Ok(mut current_qr) = data.qr.lock() 
        else {return HttpResponse::InternalServerError().finish()};
    
    match &*current_qr {
        Some(qrcode) => {
            if qrcode.is_used() {
                *current_qr = Some(CustomerQrCode::new())
            }
        }
        None => *current_qr = Some(CustomerQrCode::new())
    };

    if let Some(qrcode) = current_qr.as_ref() {

        let response = CodeResponse {
            code: qrcode.code.clone(),
        };

        return HttpResponse::Ok().json(response)
    }

    HttpResponse::InternalServerError().finish()
}



pub async fn claim_code(claim: web::Json<ClaimRequest>, data: AppData) -> HttpResponse {

    let card_id = UserId(claim.id.clone());
    let mut current_qr = data.qr.lock().unwrap();

    match &*current_qr {
        Some(qrcode) => {

            if qrcode.code != claim.code {
                warn!("Card '{}' tried to claim code '{}' but it does not match the current active code!", claim.id, claim.code);
                return HttpResponse::BadRequest().body("Invalid code!")
            }

            let mut tracker = data.cards.lock().unwrap();
            _ = tracker.stamp_card(&card_id).await;
            *current_qr = None;

            info!("Card '{}' has claimed code ''{}'", claim.id, claim.code);
            return HttpResponse::Ok().finish()
        }
        None => {
            warn!("Card '{}' tried to claim code '{}' but it there is no active qr code!", claim.id, claim.code);
            HttpResponse::BadRequest().body("Invalid code!")
        }
    }
}
