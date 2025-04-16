use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Serialize, FromRow)]
pub struct Key {
    pub id: i32,
    pub key_value: String,
    pub key_name: String,
    pub key_type: String,
    pub key_description: Option<String>,
    pub expiration_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime
}

#[derive(Deserialize)]
pub struct KeyRequest {
    pub key_value: String,
    pub key_name: String,
    pub key_type: String,
    pub key_description: Option<String>,
    pub expiration_date: Option<NaiveDateTime>
}

#[derive(Deserialize)]
pub struct PartialKeyRequest {
    pub key_value: Option<String>,
    pub key_name: Option<String>,
    pub key_type: Option<String>,
    pub key_description: Option<String>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct KeysResponse {
    pub message: String,
}
