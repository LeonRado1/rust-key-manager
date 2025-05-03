use std::env;
use lettre::transport::smtp::commands::Auth;
use sqlx::{PgPool, Row};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{Request, State};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation};
use chrono::{Duration, Utc};
use rocket::request::{FromRequest, Outcome};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::middleware::LoggedUser;
use crate::models::*;
use crate::services::enqueue_email;
use crate::utils::jwt_token::generate_jwt_token;

#[post("/login", data = "<request_data>")]
async fn login(
    pool: &State<PgPool>,
    request_data: Json<LoginRequest>
) -> Result<Json<AuthResponse>, Status> {
    let record = sqlx::query(
        "SELECT id, username, password_hash, email FROM users WHERE email = $1"
    )
    .bind(&request_data.email)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::Unauthorized)?;

    let is_valid_password = verify(&request_data.password, record.get("password_hash"))
        .map_err(|_| Status::InternalServerError)?;
    
    if !is_valid_password { 
        return Err(Status::Unauthorized); 
    }

    let token = generate_jwt_token(record.get("id"))
        .map_err(|_| Status::InternalServerError)?;

    let user = User {
        id: record.get("id"),
        username: record.get("username"),
        email: record.get("email"),
    };
    Ok(Json(AuthResponse {
        user,
        token
    }))
}

#[post("/register", data = "<request_data>")]
async fn register(
    pool: &State<PgPool>,
    request_data: Json<RegisterRequest>
) -> Result<Json<AuthResponse>, Status> {

    let password_hash = hash(&request_data.password, DEFAULT_COST)
        .map_err(|_| { Status::InternalServerError })?;

    let record = sqlx::query(
        "
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email
        "
    )
    .bind(&request_data.username)
    .bind(&request_data.email)
    .bind(&password_hash)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| {
        match e {
            sqlx::Error::Database(db_err) if db_err.code() == Some("23505".into()) => {
                Status::Conflict
            }
            _ => Status::InternalServerError
        }
    })?;

    let token = generate_jwt_token(record.get("id"))
        .map_err(|_| Status::InternalServerError)?;
    let username: String = record.get("username");

    let user = User {
        id: record.get("id"),
        username: record.get("username"),
        email: record.get("email"),
    };
    Ok(Json(AuthResponse {
        user,
        token
    }))
}

/// Request to reset password. Mail the reset token to the user email.
/// Also, create a new token in the database.
/// Body:
/// - `email`: email of the user
#[post("/password/reset-request", data = "<request_data>")]
async fn request_to_reset_password(
    pool: &State<PgPool>,
    request_data: Json<ResetPasswordRequest>
) -> Result<Json<Message>, Status> {
    // Try to find user by email
    let user = sqlx::query(
        "SELECT id FROM users WHERE email = $1"
    )
    .bind(&request_data.email)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if let Some(row) = user {
        let user_id: i32 = row.get("id");
        // Remove any existing reset tokens for the user, to avoid duplicates
        sqlx::query(
            "DELETE FROM password_reset_tokens WHERE user_id = $1"
        )
        .bind(user_id)
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
        .bind(user_id)
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
        enqueue_email(smtp_username, recipient_email, reset_token).await;

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
