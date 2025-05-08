use serde::{Serialize, Deserialize};
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
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}


#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub email: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangeUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}
