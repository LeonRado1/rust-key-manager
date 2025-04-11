use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::PgPool;
use crate::routes::auth::LoggedUser;

/// TODO:
/// - Add password, email validation.
/// - Implement tools for generation or encryption passwords, rotation.
/// - Implement users management routes, delete user, etc.
/// - Automatically analyze of relevance of users keys
/// - Implement protections: CSRF, brut-force, http, requests, ...

use crate::models::{UpdateUserRequest, UpdateUserResponse};

/// Check if the user with given id exists in the database.
pub async fn check_user_exists(pool: &PgPool, user_id: i32) -> Result<(), Status> {
    let user_exists = sqlx::query("SELECT 1 FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            Status::InternalServerError
        })?;

    if user_exists.is_none() {
        return Err(Status::Unauthorized);
    }

    Ok(())
}

/// Update user email
/// # Body Parameters:
/// - `email`: new email
#[patch("/email", data = "<request_data>")]
async fn change_email(
    pool: &State<PgPool>,
    request_data: Json<UpdateUserRequest>,
    auth: LoggedUser
) -> Result<Json<UpdateUserResponse>, Status> {
    let new_email = match &request_data.email {
        Some(email) => email,
        None => {
            eprintln!("Email is required");
            return Err(Status::BadRequest);
        }
    };

    // Check if the user with given gmail exists(email was taken by another user)
    let email_exists = sqlx::query(
        "SELECT 1
         FROM users
         WHERE email = $1 AND id != $2"
    )
    .bind(new_email)
    .bind(auth.0)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if email_exists.is_some() { return Err(Status::Conflict); }

   // Update email
    let updated = sqlx::query!(
        "UPDATE users SET email = $1 WHERE id = $2",
        new_email,
        auth.0
    )
    .execute(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if updated.rows_affected() == 0 { return Err(Status::NotFound); }

    Ok(Json(UpdateUserResponse {
        message: "Email updated successfully".to_string(),
    }))
}

/// Update user username
/// # Body Parameters:
/// - `username`: new username
#[patch("/username", data = "<request_data>")]
async fn change_username(
    pool: &State<PgPool>,
    request_data: Json<UpdateUserRequest>,
    auth: LoggedUser
) -> Result<Json<UpdateUserResponse>, Status> {
    let new_username = match &request_data.username {
        Some(username) => username,
        None => {
            eprintln!("Username is required");
            return Err(Status::BadRequest);
        }
    };

    let username_exists = sqlx::query(
        "SELECT 1
         FROM users
         WHERE username = $1 AND id != $2"
    )
    .bind(new_username)
    .bind(auth.0)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if username_exists.is_some() { return Err(Status::Conflict); }

    // Update username
    let updated = sqlx::query!(
        "UPDATE users SET username = $1 WHERE id = $2",
        new_username,
        auth.0
    )
    .execute(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if updated.rows_affected() == 0 { return Err(Status::NotFound); }

    Ok(Json(UpdateUserResponse {
        message: "Username updated successfully".to_string(),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        change_email,
        change_username
    ]
}
