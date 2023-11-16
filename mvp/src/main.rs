use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use actix_web::web::Redirect;

use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};
use tera::{Tera, Context};
use serde::Deserialize;
use rand::{distributions::Alphanumeric, Rng};
use log::debug;

type AppData = web::Data<AppState<MemoryStampCardTracker>>;

struct AppState<T>
where T: StampCardTracker
{
    app_name: String,
    tmpl: Tera,
    tracker: Mutex<T>,
}

trait StampCardTracker {
    fn get_or_create_card(&mut self, card_id: &StampCardId) -> &BasicStampCard;
    fn stamp_card(&mut self, card_id: &StampCardId) -> &BasicStampCard;
}

struct MemoryStampCardTracker {
    store: HashMap<StampCardId, BasicStampCard>
}

impl MemoryStampCardTracker {
    fn new() -> Self { // could this be default?
        MemoryStampCardTracker{
            store: HashMap::new() // interiror mutability?
        }
    }
}

impl StampCardTracker for MemoryStampCardTracker {
    fn get_or_create_card(&mut self, card_id: &StampCardId) -> &BasicStampCard {
        if !self.store.contains_key(&card_id) {
            self.store.insert(card_id.clone(), BasicStampCard::new());
        }

        self.store.get_key_value(&card_id).unwrap().1
    }

    fn stamp_card(&mut self, card_id: &StampCardId) -> &BasicStampCard {
        if !self.store.contains_key(&card_id) {
            let new_card = BasicStampCard::new();
            self.store.insert(card_id.clone(), new_card);
            return self.store.get_key_value(&card_id).unwrap().1
        }

        let existing_card = self.store.get_key_value(&card_id).unwrap().1;
        let updated_card = BasicStampCard{
            stamps: existing_card.stamps + 1,
            capacity: existing_card.capacity,
        };
        self.store.insert(card_id.clone(), updated_card);
        return self.store.get_key_value(&card_id).unwrap().1
    }
}

trait StampCard {
    fn add_stamp(&self); // result?
}
#[derive(Hash, Eq, PartialEq, Clone)]
struct StampCardId(String);

#[derive(Debug)]
struct BasicStampCard {
    stamps: u32,
    capacity: u32,
}

impl BasicStampCard {
    fn new() -> BasicStampCard {
        BasicStampCard {
            stamps: 1,
            capacity: 10
        }
    }

    fn stamp(&mut self) {
        self.stamps += 1;
    }
}





#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/qr")]
async fn qr(data: AppData) -> impl Responder {
    let code_id = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect::<String>();
    let url = format!("http://127.0.0.1:8080/stamp/{code_id}");
    let code = QrCode::new(url.as_bytes()).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    let mut ctx = Context::new();
    ctx.insert("qr", &image);
    ctx.insert("url", &url);

    let rendered = data.tmpl.render("qr.html", &ctx).unwrap();

    HttpResponse::Ok().body(rendered)
}

#[get("/stamp/{code}")]
async fn stamp(path: web::Path<String>, data: AppData) -> HttpResponse {
    let code = path.into_inner();
    let mut ctx = Context::new();

    ctx.insert("code", &code);
    let rendered = data.tmpl.render("stamp.html", &ctx).unwrap();

    HttpResponse::Ok().body(rendered)
}

#[derive(Deserialize)]
struct FormData {
    phone_number: String, // 11 ints?
}

#[post("/stamp/{code}")]
async fn submit(path: web::Path<String>, data: web::Data<AppState<MemoryStampCardTracker>>, form: web::Form<FormData>) -> impl Responder {
    let code_id = path.into_inner(); //  TODO this should trigger a refresh of the QR code
    let user_id = &form.phone_number;
    let card_id = StampCardId(user_id.clone());

    let mut tracker = data.tracker.lock().unwrap();
    let user_card = tracker.stamp_card(&card_id);
    let url = format!("http://127.0.0.1:8080/card/{}", user_id);

    dbg!(user_card);
    Redirect::to(url).temporary().see_other()
}

#[get("/card/{id}")]
async fn display_card(path: web::Path<String>, data: AppData) -> HttpResponse {
    let user_id = path.into_inner();
    let card_id = StampCardId(user_id.clone());

    let mut tracker = data.tracker.lock().unwrap();
    let card = tracker.get_or_create_card(&card_id);

    //Ok(format!("Your card has {}/{} stamps", card.stamps, card.capacity))

    let mut ctx = Context::new();
    ctx.insert("count", &card.stamps);
    let rendered = data.tmpl.render("card.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // each thread with create its own instance of HttpServer so shared state needs to be instantiated outside of this factory
    let tera = Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
    ).unwrap();
    let memory_tracker = MemoryStampCardTracker::new();


    let app_data = web::Data::new(AppState{
        app_name: String::from("foo"),
        tmpl: tera,
        tracker: Mutex::new(memory_tracker)
    });


    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(hello)
            .service(echo)
            .service(qr)
            .service(stamp)
            .service(submit)
            .service(display_card)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}