use std::env;
use sqlx::PgPool;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{Request, State};
use bcrypt::verify;
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation};
use chrono::{Duration, Utc};
use rocket::request::{FromRequest, Outcome};
use serde::{Serialize, Deserialize};

use crate::models::{User, Claims};

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

pub struct LoggedUser(pub i32);

/// https://github.com/dani-garcia/vaultwarden/blob/025bb90f8f7d4f84e8e78d14be7314ac2ab3a2fc/src/error.rs#L295
/// https://github.com/dani-garcia/vaultwarden/blob/main/src/auth.rs
/// https://github.com/dani-garcia/vaultwarden/blob/main/src/api/core/public.rs
/// Check Outcome and from_request from from_request.rs.
/// Validate JWT token
#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedUser {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => return Outcome::Error((Status::InternalServerError, "JWT_SECRET is not set"))
        };

        let auth_header = request.headers().get_one("Authorization");
        let access_token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..], // Extract token
            Some(_) => return Outcome::Error((Status::BadRequest,  "Invalid Authorization header format")),
            _ => {
                return Outcome::Error((Status::Unauthorized, "Authorization header not found"));
            }
        };

        match validate_jwt(access_token, &jwt_secret).await {
            Ok(claims) => Outcome::Success(LoggedUser(claims.sub)),
            Err(_) => Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
        }
    }
}

pub async fn validate_jwt(token: &str, secret: &str) -> Result<Claims, Status> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let token_data = decode::<Claims>(token, &decoding_key, &Validation::default())
        .map_err(|e| {
            #[cfg(debug_assertions)]
            eprintln!("JWT decoding failed: {:?}", e);
            Status::Unauthorized
        })?;

    if token_data.claims.exp < Utc::now().timestamp() as usize {
        #[cfg(debug_assertions)]
        eprintln!("JWT has expired");
        return Err(Status::Unauthorized);
    }
    Ok(token_data.claims)
}

/// Source:
/// https://github.com/fastly/pushpin/blob/main/src/core/jwt.rs
/// https://github.com/MaterializeInc/materialize/blob/main/src/frontegg-mock/src/utils.rs
/// Login user and return JWT token
#[post("/login", data = "<login_data>")]
async fn login(
    pool: &State<PgPool>,
    login_data: Json<LoginRequest>
) -> Result<Json<LoginResponse>, Status> {
    // Get user data by email
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE email = $1"
    )
    .bind(&login_data.email)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::Unauthorized)?; // 401

    // Validate password
    let is_valid_password = verify(&login_data.password, &user.password_hash)
        .map_err(|_| Status::InternalServerError)?;
    if !is_valid_password { return Err(Status::Unauthorized); }

    // Generate JWT token
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(6))
        .expect("valid timestamp")
        .timestamp() as usize;

    let token = encode(
        &Header::default(), // Use default algorithm
        &Claims {
            sub: user.id,
            exp: expiration
        },// Secret key
        &EncodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET not set").as_ref())
    ).map_err(|_| Status::InternalServerError)?;

    Ok(Json(LoginResponse { token }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![login]
}
