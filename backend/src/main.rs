#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions, AllowedHeaders};
use routes::*;
use services::*;

mod models;
mod routes;
mod services;
mod middleware;
mod utils;

#[get("/")]
async fn index() -> String {
    "Hello, rust-key-manager!".to_string()
}

fn create_db_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect_lazy(database_url)
        .expect("Failed to create pool")
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();
    
    init_email_service();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = create_db_pool(&database_url);

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Patch, Method::Delete, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error creating CORS fairing");
    
    rocket::build()
        .attach(rocket::shield::Shield::default())
        .attach(cors)
        .mount("/", routes![index])
        .mount("/auth", auth::routes())
        .mount("/users", users::routes())
        .mount("/keys", keys::routes())
        .manage(pool)
}
