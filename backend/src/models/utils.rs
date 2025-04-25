use rocket::serde::Serialize;
use rocket::serde::Deserialize;

#[derive(Serialize)]
pub struct GeneratePasswordResponse {
    pub password: String
}

#[derive(Deserialize)]
pub struct GeneratePasswordRequest {
    pub length: usize,
    pub include_special_symbols: bool,
    pub include_numbers: bool,
    pub include_uppercase: bool,
    pub include_lowercase: bool
}