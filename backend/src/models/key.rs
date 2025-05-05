use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;
use rocket::fs::TempFile;

#[derive(Serialize, FromRow)]
pub struct Key {
    pub id: i32,
    pub key_name: String,
    pub key_value: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_type: String,
    pub key_tag: Option<String>,
    pub key_pair_id: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, FromRow)]
pub struct PartialKey {
    pub id: i32,
    pub key_name: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_type: String,
    pub key_tag: Option<String>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct KeyRequest {
    pub key_name: String,
    pub key_value: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_tag: Option<String>,
    pub expiration_date: Option<String>,
}

#[derive(FromForm)]
pub struct ImportKeyForm<'r> {
    pub json: String,
    pub file: TempFile<'r>,
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
