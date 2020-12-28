use actix_web::*;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Serialize, Deserialize};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::borrow::{BorrowMut, Borrow};
use std::sync::Mutex;
use kv::*;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');


pub fn construct_google_search_url(query: &str) -> String {
    let encoded_query = utf8_percent_encode(query, FRAGMENT)
        .to_string();
    let google_search_url = format!("https://google.com/search?q={}",
                                    encoded_query);
    google_search_url
}

fn get_db_conn_string() -> String {
    let home_dir = std::env::var("HOME").unwrap_or_else(|err| {
        panic!("could not find the variable{}: {}", "HOME", err)
    });
    let database_dir = std::env::var("DATABASE_FILE").expect("Missing database location");
    let connection_string = home_dir.clone() + "/" + &database_dir;
    connection_string
}

fn get_db_instance(db_location: String) -> Store {
    let mut cfg = Config::load(db_location.clone());
    match cfg {
        Ok(cfg) => Store::new(cfg).unwrap(),
        Err(_) => {
            let mut cfg = Config::new(db_location);
            Store::new(cfg).unwrap()
        }
    }
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub fn get_command_from_query_string(query_string: &str) -> &str {
    if query_string.contains(' '){
        let space_index = query_string.find(' ').unwrap_or(0);
        return &query_string[..space_index];
    }
    &query_string
}

#[derive(Deserialize)]
pub struct SearchBy {
    query: String,
}

async fn search(query_string: web::Query<SearchBy>) -> HttpResponse {
    let cmd = get_command_from_query_string(&query_string.query);
    let redirect_url = match cmd {
        _ => construct_google_search_url(&query_string.query),
    };
    HttpResponse::Found().header("Location", redirect_url).finish()
}

#[derive(Deserialize)]
pub struct BookMark {
    key: String,
    value: String,
}

async fn add_bookmark(query_string: web::Query<SearchBy>, database: web::Data<Mutex<Store>>) -> HttpResponse {
   /* let mut db = database.lock().unwrap();
    let bookmarks = db.bucket::<String, String>(Some("bookmarks"));
    match bookmarks {
        Ok(bookmarks) => {
            bookmarks.set("test", "123");
            HttpResponse::Ok().body("inserted")
        },
        Err(_) => {
            HttpResponse::Ok().body("not inserted")
        }
    }*/

    HttpResponse::Ok().body("not inserted")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();


    let db_location = get_db_conn_string();
    let db = get_db_instance(db_location.clone());
    let data = web::Data::new(Mutex::new(db));
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .service(web::resource("search")
                .route(web::get().to(search)))
            .service(web::resource("add")
                .route(web::get().to(add_bookmark)))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
