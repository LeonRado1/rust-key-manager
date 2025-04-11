use serde::{Serialize, Deserialize};
// use uuid::Uuid; //TODO implement uuid?
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip)] // Skip it for serialization
    pub password_hash: String,
    pub created_at: NaiveDateTime
}

#[derive(Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>
}

#[derive(Serialize)]
pub struct UpdateUserResponse {
    pub message: String,
}