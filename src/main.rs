mod db;

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use db::Table;
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
    let table = Table::new("user_emails");

    table
        .insert(&connection, &key, &value)
        .map_err(|_| ErrorBadRequest("Unable to insert values"))?;

    Ok(HttpResponse::Ok().body("Beans"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(insert))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
