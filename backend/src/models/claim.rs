use rocket::serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct RecoveryCode {
    pub id: i32,
    pub user_id: i32,
    pub code: String,
    pub is_used: bool,
}
