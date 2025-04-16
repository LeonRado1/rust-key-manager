use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    pub id: i32,
    pub user_id: i32,
    #[serde(skip)]
    pub reset_token: String,
    pub expiration_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct ResetData {
    pub reset_token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ResetResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct Message {
    pub content: String
}
