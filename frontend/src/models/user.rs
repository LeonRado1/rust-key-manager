use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub email: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChangeUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}
