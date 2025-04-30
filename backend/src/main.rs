#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

mod models;
mod routes;
mod services;

use routes::*;
use services::*;

#[derive(Serialize)]
struct Message {
    content: String,
}

#[get("/")]
async fn index() -> Json<Message> {
    Json(Message { content: "Hello, rust-key-manager!".to_string() })
}

fn create_db_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(database_url)
        .expect("Failed to create pool")
}

#[launch]
async fn rocket() -> _ {
    // Load environment variables from the .env file
    dotenv::dotenv().ok();

    // Initialize the email service to process email requests
    init_email_reset_password_service();
    init_email_service();
    // Use enqueue_email to send emails

    // Get the database connection string from environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a connection pool to the database
    let pool = create_db_pool(&database_url);

    // check_key_relevance(&pool).await.unwrap();
    // Check key relevance and send emails if needed
    if let Err(e) = check_keys_relevance(&pool).await {
        eprintln!("Error checking key relevance: {:?}", e);
    }

    // Initialize Rocket and routes
    rocket::build()
        .attach(rocket::shield::Shield::default()) // Mechanism for configuring HTTP security headers
        .mount("/", routes![index])
        .mount("/auth", auth::routes())
        .mount("/users", users::routes())
        .mount("/keys", keys::routes())
        .mount("/status", health::routes())
        .mount("/utils", utils::routes())
        // .mount("/docs", routes::docs::routes())
        .manage(pool)
}
