use std::env;
use sqlx::PgPool;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{Request, State};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation};
use chrono::{Duration, Utc};
use garde::Validate;
use rocket::request::{FromRequest, Outcome};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{User, NewUser, Claims, ResetPasswordRequest, PasswordResetToken, ResetData, ResetResponse, Message, ValidateEmail, ValidatePassword, ValidateUsername};
/// To send email
use crate::services::enqueue_email_reset_password;

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


/// https://github.com/sfackler/rust-postgres/blob/master/tokio-postgres/src/error/sqlstate.rs
#[post("/register", data = "<request_data>")]
async fn register(
    pool: &State<PgPool>,
    request_data: Json<NewUser>
) -> Result<Json<User>, Status> {
    // validate email
    if ValidateEmail::validate(
        &ValidateEmail { email: request_data.email.clone(), }
    ).is_err() { return Err(Status::BadRequest); }

    // Check if user with given email already exists
    let user_exists = sqlx::query(
        "SELECT 1 FROM users WHERE email = $1"
    )
    .bind(&request_data.email)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| { Status::InternalServerError })?;
    if user_exists.is_some() { return Err(Status::Conflict); }// 409

    // Check user password
    if ValidatePassword::validate(
        &ValidatePassword { password: request_data.password.to_string(), }
    ).is_err() { return Err(Status::BadRequest); }

    // Validate username
    if ValidateUsername::validate(
        &ValidateUsername { username: request_data.username.to_string(), }
    ).is_err() { return Err(Status::BadRequest); }

    // Hash user password using bcrypt
    let password_hash = hash(&request_data.password, DEFAULT_COST)
        .map_err(|_| { Status::InternalServerError })?;

    // Try to create a user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $2, $3)
             RETURNING id, username, email, password_hash, created_at"
    ).bind(&request_data.username)
        .bind(&request_data.email)
        .bind(&password_hash)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e); // Log occurred error
            match e {
                sqlx::Error::Database(db_err) if db_err.code() == Some("23505".into()) => {
                    Status::Conflict // Unique violation
                }
                _ => Status::InternalServerError
            }
        })?;

    Ok(Json(user))
}


/// Request to reset password. Mail the reset token to the user email.
/// Also create a new token in the database.
/// Body:
/// - `email`: email of the user
#[post("/password/reset-request", data = "<request_data>")]
async fn request_to_reset_password(
    pool: &State<PgPool>,
    request_data: Json<ResetPasswordRequest>
) -> Result<Json<Message>, Status> {
    // Try to find user by email
    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        request_data.email
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if let Some(user) = user {
        // Remove any existing reset tokens for the user, to avoid duplicates
        sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE user_id = $1",
            user.id
        )
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

        // `new_v4` - optimized method for generating random UUIDs
        let reset_token = Uuid::new_v4().to_string();
        let expiration_date = Utc::now() + Duration::hours(1);

        // Save token to database
        sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, reset_token, expiration_date)
         VALUES ($1, $2, $3)"
        )
        .bind(user.id)
        .bind(&reset_token)
        .bind(expiration_date)
        .execute(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            Status::InternalServerError
        })?;

        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        // Clone email to allow move it to the async block
        let recipient_email = request_data.email.clone();
        // Param:
        // - `smtp_username`: Sender email
        // - `request_data.email`: Recipient email
        // - `reset_token`: Reset token to reset password
        enqueue_email_reset_password(smtp_username, recipient_email, reset_token).await;

        Ok(Json(Message {
            content: "Password reset email sent.".to_string(),
        }))
    } else {
        Err(Status::NotFound)
    }
}


/// Reset user password using reset token.
/// Body:
/// - `reset_token`: reset token
/// - `new_password`: new password
/// - `email`: email of the user
#[post("/password/reset", data = "<request_data>")]
pub async fn reset_password(
    pool: &State<PgPool>,
    request_data: Json<ResetData>
) -> Result<Json<ResetResponse>, Status> {
    // Retrieve token from database
    let reset_record = sqlx::query_as::<_, PasswordResetToken>(
        "SELECT id, user_id, reset_token, expiration_date, created_at
         FROM password_reset_tokens WHERE reset_token = $1"
    )
    .bind(&request_data.reset_token)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if let Some(token) = reset_record {
        // Check token for expiration
        if token.expiration_date < Utc::now().naive_utc() {
            return Err(Status::BadRequest);
        }

        // Token is valid, then update user password.
        // First, hash user password
        let hashed_password = hash(&request_data.new_password, DEFAULT_COST)
            .map_err(|_| Status::InternalServerError)?;

        // Update user password in database
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(&hashed_password)
            .bind(token.user_id)
            .execute(pool.inner())
            .await
            .map_err(|e| {
                eprintln!("Database error: {:?}", e);
                Status::InternalServerError
            })?;

        // Delete token from database
        sqlx::query("DELETE FROM password_reset_tokens WHERE reset_token = $1")
            .bind(&request_data.reset_token)
            .execute(pool.inner())
            .await
            .map_err(|e| {
                eprintln!("Database error: {:?}", e);
                Status::InternalServerError
            })?;

        // Create notification in db system
        sqlx::query("INSERT INTO notifications (
           user_id,
           notification_type,
           message
        ) VALUES ($1, $2)")
        .bind(token.user_id)
        .bind("password_reset")
        .bind("Password reset successfully")
        .execute(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            Status::InternalServerError
        })?;

        Ok(Json(ResetResponse {
            message: "Password reset successfully".to_string(),
        }))
    } else {
        Err(Status::NotFound)
    }
}


pub fn routes() -> Vec<rocket::Route> {
    routes![
        login,
        register,
        request_to_reset_password,
        reset_password
    ]
}
