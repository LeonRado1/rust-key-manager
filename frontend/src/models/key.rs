use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PartialKey {
    pub id: i32,
    pub key_name: String,
    pub key_description: Option<String>,
    pub key_type_id: i32,
    pub key_type: String,
    pub key_tag: Option<String>,
    pub key_pair_id: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}