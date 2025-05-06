use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    pub id: i32,
    pub key_name: String,
    pub key_value: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_type: String,
    pub key_tag: Option<String>,
    pub key_pair_value: Option<String>,
    pub expiration_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub is_revoked: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PartialKey {
    pub id: i32,
    pub key_name: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_type: String,
    pub key_tag: Option<String>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct KeyRequest {
    pub key_name: String,
    pub key_value: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_tag: Option<String>,
    pub expiration_date: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UpdateKeyRequest {
    pub new_password: Option<String>,
    pub new_expiration_date: Option<String>,
}
