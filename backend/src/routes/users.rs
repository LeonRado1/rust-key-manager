use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::PgPool;
use crate::routes::auth::LoggedUser;

use garde::Validate;

/// TODO:
/// - Add validation.
/// - Implement tools for generation or encryption passwords, `rotation`.
/// - Automatically analyze of relevance of users keys
/// - Implement protections: CSRF, brut-force, http, requests, ...

use crate::models::{
    UpdateUserRequest, UpdateUserResponse, DeleteUserRequest,
    ValidateEmail, ValidatePassword, ValidateUsername
};

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

    // Validate email
    if ValidateEmail::validate(
        &ValidateEmail { email: new_email.clone(), }
    ).is_err() { return Err(Status::BadRequest); }

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
    // Check if username is provided in the request
    let new_username = match &request_data.username {
        Some(username) => username,
        None => { return Err(Status::BadRequest); }
    };

    // Validate username
    if ValidateUsername::validate(
        &ValidateUsername { username: new_username.to_string(), }
    ).is_err() { return Err(Status::BadRequest); }

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

#[patch("/password", data = "<request_data>")]
async fn change_password(
    pool: &State<PgPool>,
    request_data: Json<UpdateUserRequest>,
    auth: LoggedUser
) -> Result<Json<UpdateUserResponse>, Status> {
    // Ensure new_password is provided
    let new_password = request_data
        .new_password
        .as_deref()
        .ok_or(Status::BadRequest)?;

    // Ensure old_password is provided
    let old_password = request_data
        .old_password
        .as_deref()
        .ok_or(Status::BadRequest)?;

    // Check new password
    if ValidatePassword::validate(
        &ValidatePassword { password: new_password.to_string(), }
    ).is_err() { return Err(Status::BadRequest); }

    // Retrieve old password from database
    let old_password_hash = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| { Status::InternalServerError })?;

    // Verify old password
    if !bcrypt::verify(old_password, &old_password_hash.password_hash).map_err(|_| Status::InternalServerError)? {
        return Err(Status::Unauthorized);
    }

    // Hash new password
    let hashed_password = bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|_| Status::InternalServerError)?;

    // Save new password in database
    let updated = sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        hashed_password,
        auth.0
    )
    .execute(pool.inner())
    .await
    .map_err(|_| { Status::InternalServerError })?;

    if updated.rows_affected() == 0 { return Err(Status::NotFound); }
    Ok(Json(UpdateUserResponse {
        message: "Password updated successfully".to_string(),
    }))
}

#[delete("/account", data = "<request_data>")]
async fn delete_user_account(
    pool: &State<PgPool>,
    request_data: Json<DeleteUserRequest>,
    auth: LoggedUser
) -> Result<Json<UpdateUserResponse>, Status> {
    let password = &request_data.password;

    // Validate password
    if ValidatePassword::validate(
        &ValidatePassword { password: password.to_string(), }
    ).is_err() { return Err(Status::BadRequest); }

    // Retrieve password from database
    let password_hash = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| { Status::InternalServerError })?;

    // Verify old password
    if !bcrypt::verify(password, &password_hash.password_hash).map_err(|_| Status::InternalServerError)? {
        return Err(Status::Unauthorized);
    }

    // Delete user from the database
    let deleted = sqlx::query!(
        "DELETE FROM users WHERE id = $1",
        auth.0
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if deleted.rows_affected() == 0 { return Err(Status::NotFound); }

    Ok(Json(UpdateUserResponse {
        message: "User account deleted successfully".to_string(),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        change_email,
        change_username,
        change_password,
        delete_user_account
    ]
}
