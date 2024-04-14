use shuttle_actix_web::ShuttleActixWeb;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, web::ServiceConfig};
use actix_web::dev::ConnectionInfo;
use actix_web::web::Redirect;
use async_trait::async_trait;

use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};
use tera::{Tera, Context};
use serde::{Deserialize, Serialize};
use rand::{distributions::Alphanumeric, Rng};
use log::{debug, info};
use mongodb::{Collection, Database};
use mongodb::{bson::{doc, Document}, options::FindOneOptions};

type AppData = web::Data<AppState<MongoDbStampCardRepository>>;

mod pages;

struct AppState<T>
where T: StampCardTracker
{
    app_name: String,
    tmpl: Tera,
    cards: Mutex<T>,
}

#[async_trait]
trait StampCardTracker {
    async fn get_or_create_card(&mut self, card_id: &UserId) -> BasicStampCard;
    async fn stamp_card(&mut self, card_id: &UserId) -> BasicStampCard;
}

struct MongoDbStampCardRepository {
    collection: Collection<BasicStampCard>
}

#[async_trait]
impl StampCardTracker for MongoDbStampCardRepository {
    async fn get_or_create_card(&mut self, user_id: &UserId) -> BasicStampCard {
        info!("Searching for card with user_id {}", user_id);
        let filter = doc! {
            "user_id": user_id.to_string()
        };
        let find = self.collection
            .find_one(filter, None)
            .await.unwrap();

        return match find {
            Some(card) => {
                info!("Found existing card for user_id {}", user_id);
                card
            },
            None => {
                info!("Creating new card for user_id {}", user_id);
                let new_card = BasicStampCard::new(user_id.clone());
                self.collection.insert_one(&new_card, None).await.unwrap(); // TODO ? operator
                new_card
            }
        }
    }

    async fn stamp_card(&mut self, user_id: &UserId) -> BasicStampCard {
        let user_card = self.get_or_create_card(user_id).await;
        let stamped_card = user_card.with_stamp();
        let filter = doc! {
            "user_id": user_id.to_string() // todo extract method
        };

        self.collection.replace_one(filter, &stamped_card, None).await.unwrap();

        info!("Card for user_id {} now has {} stamps", user_id, stamped_card.stamps);
        stamped_card
    }
}

// struct MemoryStampCardTracker {
//     store: HashMap<UserId, BasicStampCard>
// }

// impl MemoryStampCardTracker {
//     fn new() -> Self { // could this be default?
//         MemoryStampCardTracker{
//             store: HashMap::new() // interiror mutability?
//         }
//     }
// }

// impl StampCardTracker for MemoryStampCardTracker {
//     fn get_or_create_card(&mut self, card_id: &UserId) -> &BasicStampCard {
//         if !self.store.contains_key(&card_id) {
//             self.store.insert(card_id.clone(), BasicStampCard::new());
//         }
//
//         self.store.get_key_value(&card_id).unwrap().1
//     }
//
//     fn stamp_card(&mut self, card_id: &UserId) -> &BasicStampCard {
//         if !self.store.contains_key(&card_id) {
//             let new_card = BasicStampCard::new();
//             self.store.insert(card_id.clone(), new_card);
//             return self.store.get_key_value(&card_id).unwrap().1
//         }
//
//         let existing_card = self.store.get_key_value(&card_id).unwrap().1;
//         let updated_card = BasicStampCard{
//             stamps: existing_card.stamps + 1,
//             capacity: existing_card.capacity,
//         };
//         self.store.insert(card_id.clone(), updated_card);
//         return self.store.get_key_value(&card_id).unwrap().1
//     }
// }

trait StampCard {
    fn add_stamp(&self); // result?
}
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
struct UserId(String);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BasicStampCard {
    user_id: UserId,
    stamps: u32,
    capacity: u32,
}

impl BasicStampCard {
    fn new(user_id: UserId) -> BasicStampCard {
        BasicStampCard {
            user_id,
            stamps: 0,
            capacity: 10
        }
    }

    fn with_stamp(&self) -> Self {
        BasicStampCard {
            user_id: self.user_id.clone(),
            stamps: self.stamps + 1, // todo! clamp at max 10, ignore any others
            capacity: self.capacity,
        }
    }
}


#[get("/qr")] // TODO only admin should be able to GET this
async fn qr(data: AppData, conn: ConnectionInfo) -> impl Responder {
    let code_id = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect::<String>();
    let url = format!("{}://{}/stamp/{}", conn.scheme(), conn.host(), code_id);
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
async fn submit(
    path: web::Path<String>,
    state: AppData,
    form: web::Form<FormData>) -> impl Responder {
    let code_id = path.into_inner(); //  TODO this should trigger a refresh of the QR code
    let user_id = &form.phone_number;
    let card_id = UserId(user_id.clone());

    let mut tracker = state.cards.lock().unwrap();
    let user_card = tracker.stamp_card(&card_id).await;
    let url = format!("/card/{}", user_id);

    Redirect::to(url).temporary().see_other()
}

#[get("/card/{id}")]
async fn display_card(path: web::Path<String>, data: AppData) -> HttpResponse {
    let user_id = path.into_inner();
    let card_id = UserId(user_id.clone());

    let mut tracker = data.cards.lock().unwrap();
    let card = tracker.get_or_create_card(&card_id).await;

    //Ok(format!("Your card has {}/{} stamps", card.stamps, card.capacity))

    let mut ctx = Context::new();
    ctx.insert("count", &card.stamps);
    let rendered = data.tmpl.render("card.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}


#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::MongoDb] db: Database
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // each thread with create its own instance of HttpServer so shared state needs to be instantiated outside of this factory

    let tera = Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
    ).unwrap();

    let mongo_repo = MongoDbStampCardRepository{
        collection: db.collection::<BasicStampCard>("cards")
    };

    let app_data = web::Data::new(AppState{
        app_name: String::from("foo"),
        tmpl: tera,
        cards: Mutex::new(mongo_repo),
    });
    
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(app_data.clone());
        cfg.service(qr);
        cfg.service(stamp);
        cfg.service(submit);
        cfg.service(display_card);
    };

    Ok(config.into())
}
