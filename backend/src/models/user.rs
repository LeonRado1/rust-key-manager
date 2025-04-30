use serde::{Serialize, Deserialize};
// use uuid::Uuid; //TODO implement uuid?
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String
}

#[derive(Deserialize)]
pub struct RegisterRequest {
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

#[derive(Serialize, Deserialize, FromRow)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub token: String
}
