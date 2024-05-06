use shuttle_actix_web::ShuttleActixWeb;
use std::fmt::{Display, Formatter};
use std::{env, io};
use std::sync::atomic::spin_loop_hint;
use std::sync::{Mutex, MutexGuard};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, web::ServiceConfig, Scope, HttpRequest};
use actix_web::cookie::time::macros::date;
use actix_web::dev::{ConnectionInfo, fn_service, ServiceRequest, ServiceResponse};
use actix_web::web::{get, post, Redirect, resource};
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

mod stampcard;
mod customer_code;
mod db;

type AppData = web::Data<State>;

struct State
{
    app_name: String,
    cards: Mutex<db::MongoDbStampCardRepository>,
    qr: Mutex<Option<CustomerQrCode>>, // TODO this could be a dictionary to allow multiple stores to display unique qr codes
}



// fn use_mutex() {
//     let data = Mutex::new(Some("data"));
//     let locked = data.lock().unwrap();
//     match &*locked {
//         Some(data) => {},
//         None => {}
//     }
// }

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

// #[derive(Deserialize)]
// struct FormData {
//     phone_number: String, // 11 ints?
// }





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

    let mongo_repo = db::MongoDbStampCardRepository{
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
                .service(resource("/customercode").route(get().to(customer_code::get_code)))
                .service(resource("/customercode/claim").route(post().to(customer_code::claim_code)))
                .service(resource("/stampcard/{id}").route(get().to(stampcard::get_card)))
                .service(resource("/stampcard/{id}/reset").route(post().to(stampcard::reset_card)))
                .wrap(Cors::permissive())
                .app_data(app_data.clone())
        );

        // TODO is this needed
        // let path = env::current_dir().unwrap();
        // println!("The current directory is {}", path.display());
        
        // serve static assets
        cfg.service(actix_files::Files::new("/", "crates/seven-oz-loyalty/assets")
            .index_file("index.html")
            .default_handler(fn_service(|req: ServiceRequest| async {
                let (req, _) = req.into_parts();
                let file = NamedFile::open_async("crates/seven-oz-loyalty/assets/index.html").await?;
                let res = file.into_response(&req);
                Ok(ServiceResponse::new(req, res))
            })));
    };

    Ok(config.into())
}
