use std::env;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use crate::models::Claims;

pub fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims, Status> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let token_data = decode::<Claims>(token, &decoding_key, &Validation::default())
        .map_err(|_| Status::Unauthorized)?;

    if token_data.claims.exp < Utc::now().timestamp() as usize {
        return Err(Status::Unauthorized);
    }

    Ok(token_data.claims)
}

pub fn generate_jwt_token(user_id: i32) -> Result<String, Status> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(6))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: format!("{}", user_id),
        exp: expiration,
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|_| Status::InternalServerError)?;

    Ok(token)
}
