#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

mod routes {
    pub mod auth;
    pub mod keys;
}

#[derive(Serialize)]
struct Message {
    content: String,
}

#[get("/")]
async fn index() -> Json<Message> {
    Json(Message { content: "Hello, rust-key-manager!".to_string(), })
}

#[launch]
async fn rocket() -> _ {
    // Load environment variables from the .env file
    dotenv::dotenv().ok();

    // Get the database connection string from environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a connection pool to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Initialize Rocket and routes
    rocket::build()
        .mount("/", routes![index])
        // .mount("/auth", routes::auth::routes()) // TODO
        // .mount("/keys", routes::keys::routes()) // TODO
        .manage(pool)
}

