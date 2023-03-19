extern crate adrodb;

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound},
    get, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use adrodb::Table;
use rusqlite::{Connection, Result};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/{key}/{value}")]
async fn insert(path: web::Path<(String, String)>) -> Result<HttpResponse, Error> {
    let (key, value) = path.into_inner();
    let connection = Connection::open("./test.sqlite")
        .map_err(|_| ErrorInternalServerError("Unable to connect to database"))?;

    Table::existing("user_emails", &connection)
        .set(&key, &value)
        .map_err(|_| ErrorBadRequest("Unable to insert values"))?;

    Ok(HttpResponse::Ok().body("Beans"))
}

#[get("/{key}")]
async fn get(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let key = path.into_inner();
    let connection = Connection::open("./test.sqlite")
        .map_err(|_| ErrorInternalServerError("Unable to connect to database"))?;

    let value = Table::existing("user_emails", &connection)
        .get::<String>(&key)
        .map_err(|_| ErrorNotFound("Value was not found"))?;

    Ok(HttpResponse::Ok().body(value))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(insert).service(get))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
