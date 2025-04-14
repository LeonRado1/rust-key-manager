use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String
}
