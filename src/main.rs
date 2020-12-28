use actix_web::*;
use serde::{Serialize, Deserialize};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use r2d2_jfs::JfsConnectionManager;
use std::fs;
use std::{collections::BTreeMap};


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
    let database_dir = std::env::var("DATABASE_DIR").expect("Missing database location");
    let connection_string = home_dir.clone() + "/" + &database_dir;
    fs::create_dir_all(connection_string.clone()).unwrap();
    connection_string
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

#[derive(Serialize,Deserialize)]
pub struct BookMark {
    key: String,
    value: String,
}
type DBPool = r2d2::Pool<JfsConnectionManager>;

async fn get_all_bookmarks(database_pool: web::Data<DBPool>) -> actix_http::error::Result<HttpResponse, Error> {
    let connection = database_pool.get()
        .map_err(|_| HttpResponse::InternalServerError().body("Empty Connection Pool"))?;

    let map: BTreeMap<String, BTreeMap<String, String>> = connection.all().unwrap();
    println!("{:?}", map);
    for (k, v) in &map {
        println!("{}", k);
        for (kk, vv) in v {
            println!("{} : {}", kk, vv);
        }
    }
    Ok(HttpResponse::Ok().body("Added value to db"))
}

async fn add_bookmark(bookmark: web::Query<BookMark>,
                      database_pool: web::Data<DBPool>) -> actix_http::error::Result<HttpResponse, Error> {
    println!("{}", bookmark.key);
    let connection = database_pool.get()
        .map_err(|_| HttpResponse::InternalServerError().body("Empty Connection Pool"))?;
    let example = BookMark { key: "badar".to_owned(), value: "ddadaad".to_owned() };

    let _save_action = web::block(move || {
        connection.save_with_id(&example, example.key.as_str())
    })
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().body("Added value to db"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let db_dir = get_db_conn_string();
    let connection_manager = JfsConnectionManager::file(db_dir + "db.json").unwrap();
    let pool = r2d2::Pool::builder().build(connection_manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(web::resource("search")
                .route(web::get().to(search)))
            .service(web::resource("add")
                .route(web::get().to(add_bookmark)))
            .service(web::resource("all")
                .route(web::get().to(get_all_bookmarks)))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
