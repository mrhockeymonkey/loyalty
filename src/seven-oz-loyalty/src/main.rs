use shuttle_actix_web::ShuttleActixWeb;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::atomic::spin_loop_hint;
use std::sync::{Mutex, MutexGuard};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, web::ServiceConfig, Scope, HttpRequest};
use actix_web::cookie::time::macros::date;
use actix_web::dev::{ConnectionInfo, fn_service, ServiceRequest, ServiceResponse};
use actix_web::web::Redirect;
use actix_files::{Files, NamedFile};
//use actix_web::error::UrlencodedError::ContentType;
// use async_trait::async_trait;
use actix_web::http::header::ContentType;
// use qrcode::render::svg;
// use qrcode::{EcLevel, QrCode, Version};
use serde::{Deserialize, Serialize};
// use rand::{distributions::Alphanumeric, Rng};
use log::{debug, info, warn};
use mongodb::{Collection, Database};
use mongodb::{bson::{doc, Document}, options::FindOneOptions};
use thiserror::Error;
use loyalty_core::{StampCardTracker, BasicStampCard, UserId, qr_gen};
use loyalty_core::qr_gen::CustomerQrCode;
use actix_cors::Cors;

type AppData = web::Data<State>;

struct State
{
    app_name: String,
    cards: Mutex<MongoDbStampCardRepository>,
    qr: Mutex<Option<CustomerQrCode>>, // TODO this could be a dictionary to allow multiple stores to display unique qr codes
}

struct MongoDbStampCardRepository {
    collection: Collection<BasicStampCard>
}

#[derive(Debug, Error)]
enum StampCardRepositoryError {
    #[error("mongodb error")]
    MongoDbError(#[from] mongodb::error::Error)
}


impl MongoDbStampCardRepository {
    async fn get_or_create_card(&mut self, user_id: &UserId) -> Result<BasicStampCard, StampCardRepositoryError> {
        info!("Searching for card with user_id {}", user_id);
        let filter = doc! {
            "user_id": user_id.to_string()
        };
        let find = self.collection
            .find_one(filter, None)
            .await?;

        match find {
            Some(card) => {
                info!("Found existing card for user_id {}", user_id);
                Ok(card)
            },
            None => {
                info!("Creating new card for user_id {}", user_id);
                let new_card = BasicStampCard::new(user_id.clone());
                self.collection.insert_one(&new_card, None).await?;
                Ok(new_card)
            }
        }
    }

    // TODO Command Query Separation
    async fn stamp_card(&mut self, user_id: &UserId) -> Result<BasicStampCard, StampCardRepositoryError> {
        let user_card = self.get_or_create_card(user_id).await?;
        let stamped_card = user_card.with_stamp();
        let filter = doc! {
            "user_id": user_id.to_string() // todo extract method
        };

        self.collection.replace_one(filter, &stamped_card, None).await?;

        info!("Card for user_id {} now has {} stamps", user_id, stamped_card.stamps);
        Ok(stamped_card)
    }
}

// fn use_mutex() {
//     let data = Mutex::new(Some("data"));
//     let locked = data.lock().unwrap();
//     match &*locked {
//         Some(data) => {},
//         None => {}
//     }
// }


#[derive(Serialize)]
struct QrResponse {
    code: String
}

#[get("/qr")]
async fn get_qr(data: AppData, conn: ConnectionInfo) -> HttpResponse {
    info!("getting QR");
    let mut current_qr = data.qr.lock().unwrap(); // TODO result?
    match &*current_qr {
        Some(qrcode) => {
            if qrcode.is_used() {
                *current_qr = Some(CustomerQrCode::new())
            }
        }
        None => *current_qr = Some(CustomerQrCode::new())
    };

    if let Some(qrcode) = current_qr.as_ref() {
        
        let response = QrResponse{
            code: qrcode.code.clone(),
        };

        return HttpResponse::Ok().json(response)
    }

    HttpResponse::InternalServerError().finish()
}
// 
// #[get("/qr")] // TODO only admin should be able to GET this
// async fn qr(data: AppData, conn: ConnectionInfo) -> HttpResponse {
// 
//     let mut current_qr = data.qr.lock().unwrap(); // TODO result?
//     match &*current_qr {
//         Some(qrcode) => {
//             if qrcode.is_used() {
//                 *current_qr = Some(new_qr(conn))
//             }
//         }
//         None => *current_qr = Some(new_qr(conn))
//     };
// 
// 
//     
//     if let Some(qrcode) = current_qr.as_ref() {
//         let image = qrcode
//             .render()
//             .min_dimensions(200, 200)
//             .dark_color(qr_gen::Color("#000000"))
//             .light_color(qr_gen::Color("#ffffff"))
//             .build();
// 
//         let mut ctx = Context::new();
//         ctx.insert("qr", &image);
//         ctx.insert("url", &qrcode.url);
// 
//         let rendered = data.tmpl.render("qr.html", &ctx).unwrap();
// 
//         return HttpResponse::Ok().body(rendered);
//     }
//     
//     HttpResponse::InternalServerError().body("") // TODO
// }

// #[get("/stamp/{code}")]
// async fn stamp(path: web::Path<String>, data: AppData) -> HttpResponse {
//     let code = path.into_inner();
//     let mut ctx = Context::new();
// 
//     ctx.insert("code", &code);
//     let rendered = data.tmpl.render("stamp.html", &ctx).unwrap();
// 
//     HttpResponse::Ok().body(rendered)
// }

#[derive(Deserialize)]
struct FormData {
    phone_number: String, // 11 ints?
}

#[derive(Deserialize, Serialize)]
struct Claim {
    id: String,
    code: String
}

#[post("/claim")]
pub async fn claim_code(claim: web::Json<Claim>, data: AppData) -> HttpResponse {
    
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

#[derive(Serialize)]
struct CardResponse {
    stamps: u32
}

#[get("/card/{id}")]
async fn get_card(path: web::Path<String>, data: AppData) -> HttpResponse {
    let user_id = path.into_inner();
    info!("GET /card/{}", user_id);
    let card_id = UserId(user_id.clone());
    let mut tracker = data.cards.lock().unwrap();
    let card = tracker.get_or_create_card(&card_id).await.unwrap();
    let response = CardResponse { stamps: card.stamps };
    HttpResponse::Ok().json(response)
}

#[get("/")]
async fn index() -> io::Result<NamedFile> {
    NamedFile::open_async("seven-oz-loyalty/assets/index.html").await
}


#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] db: Database
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // each thread with create its own instance of HttpServer so shared state needs to be instantiated outside of this factory
    let cors = Cors::default()
        .allowed_origin("http://localhost:8081")
        // .allowed_origin_fn(|origin, _req_head| {
        //     origin.as_bytes().ends_with(b".rust-lang.org")
        // })
        .allowed_methods(vec!["GET", "POST"])
        // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        // .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600);

    let mongo_repo = MongoDbStampCardRepository{
        collection: db.collection::<BasicStampCard>("cards")
    };

    let app_data = web::Data::new(State {
        app_name: String::from("foo"),
        cards: Mutex::new(mongo_repo),
        qr: Mutex::new(None)
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            Scope::new("/api")
                .service(get_qr)
                .service(claim_code)
                .service(get_card)
                .wrap(Cors::permissive())
                .app_data(app_data.clone())
        );
        // cfg.app_data(app_data.clone());
        // cfg.service(get_qr);
        // cfg.service(stamp);
        // cfg.service(submit);
        // cfg.service(display_card);
        // cfg.service(index);
        
        cfg.service(actix_files::Files::new("/", "seven-oz-loyalty/assets")
            .index_file("index.html")
            .default_handler(fn_service(|req: ServiceRequest| async {
                let (req, _) = req.into_parts();
                let file = NamedFile::open_async("seven-oz-loyalty/assets/index.html").await?;
                let res = file.into_response(&req);
                Ok(ServiceResponse::new(req, res))
            })));
        
        // TODO can this point to client project?
            // .show_files_listing()
            // .index_file("index.html"));
        // cfg.service(web::resource("/path1").to(|| HttpResponse::Ok()));
        // cfg.service(
        //     web::scope("")
        //         .service(get_qr)
        // );
    };

    Ok(config.into())
}
