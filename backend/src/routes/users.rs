use rocket::serde::json::Json;
use sqlx::PgPool;
use rocket::http::Status;
use rocket::State;
use bcrypt::{hash, DEFAULT_COST};

use crate::models::{User, NewUser};

/// https://github.com/sfackler/rust-postgres/blob/master/tokio-postgres/src/error/sqlstate.rs
#[post("/", data = "<request_data>")]
async fn create_user(pool: &State<PgPool>, request_data: Json<NewUser>) -> Result<Json<User>, Status> {
    // Hash user password using bcrypt
    let password_hash = hash(&request_data.password, DEFAULT_COST)
        .map_err(|_| { Status::InternalServerError })?;

    // Try to create a user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $2, $3)
             RETURNING id, username, email, password_hash, created_at"
    ).bind(&request_data.username)
        .bind(&request_data.email)
        .bind(&password_hash)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e); // Log occurred error
            match e {
                sqlx::Error::Database(db_err) if db_err.code() == Some("23505".into()) => {
                    Status::Conflict // Unique violation
                }
                _ => Status::InternalServerError
            }
        })?;

    Ok(Json(user))
}




pub fn routes() -> Vec<rocket::Route> {
    routes![create_user]
}
