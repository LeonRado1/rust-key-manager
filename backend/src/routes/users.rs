use bcrypt::verify;
use chrono::Local;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::{PgPool, Row};
use crate::middleware::LoggedUser;
use crate::models::{AuthResponse, ChangeUserRequest, User};
use crate::services::{enqueue_email, EmailRequest, GLOBAL_SENDER_EMAIL};
use crate::utils::jwt_token::generate_jwt_token;
use crate::utils::validation::{validate_email, validate_username};

#[get("/")]
async fn get_current_user(
    pool: &State<PgPool>,
    auth: LoggedUser
) -> Result<Json<AuthResponse>, Status> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email FROM users WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::NotFound)?;

    let token = generate_jwt_token(user.id)
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Json(AuthResponse { user, token }))
}

#[put("/", data = "<request_data>")]
async fn change_user(
    pool: &State<PgPool>,
    request_data: Json<ChangeUserRequest>,
    auth: LoggedUser
) -> Result<Json<User>, Status> {
    let record = sqlx::query!(
        "SELECT id, username, email, password_hash
         FROM users
         WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::NotFound)?;

    let is_valid_password = verify(&request_data.password, &record.password_hash)
        .map_err(|_| Status::InternalServerError)?;

    if !is_valid_password {
        return Err(Status::Unauthorized);
    }
    
    let email = request_data.email.clone();
    let username = request_data.username.clone();
    
    if let Some(new_email) = &email {
        if let Err(_e) = validate_email(&new_email) {
            return Err(Status::ExpectationFailed);
        };
        
        if new_email != &record.email {
            let email_exists = sqlx::query(
                "SELECT 1 FROM users WHERE email = $1 AND id != $2"
            )
            .bind(new_email)
            .bind(auth.0)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;

            if email_exists.is_some() {
                return Err(Status::Conflict);
            }
        }
        else {
            return Err(Status::Conflict);
        }
    }
    
    if let Some(new_username) = &username {
        if let Err(_e) = validate_username(&new_username) {
            return Err(Status::ExpectationFailed);
        };
        
        if new_username != &record.username {
            let username_exists = sqlx::query(
                "SELECT 1 FROM users WHERE username = $1 AND id != $2"
            )
                .bind(new_username)
                .bind(auth.0)
                .fetch_optional(pool.inner())
                .await
                .map_err(|_| Status::InternalServerError)?;

            if username_exists.is_some() {
                return Err(Status::Conflict);
            }
        }
        else {
            return Err(Status::Conflict);
        }
    }

    let updated_user = sqlx::query_as!(
        User,
        "UPDATE users 
         SET 
             username = COALESCE($1, username),
             email = COALESCE($2, email)
         WHERE id = $3
         RETURNING id, username, email",
        username,
        email,
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(updated_user))
}

#[delete("/", data = "<request_data>")]
async fn delete_user(
    pool: &State<PgPool>,
    request_data: Json<ChangeUserRequest>,
    auth: LoggedUser
) -> Result<(), Status> {
    let record = sqlx::query!(
        "SELECT id, username, email, password_hash
         FROM users
         WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::NotFound)?;

    let is_valid_password = verify(&request_data.password, &record.password_hash)
        .map_err(|_| Status::InternalServerError)?;

    if !is_valid_password {
        return Err(Status::Unauthorized);
    }

    let mut tx = pool.inner()
        .begin()
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    sqlx::query!(
        "DELETE FROM recovery_codes 
         WHERE user_id = $1",
        auth.0
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;
    
    sqlx::query!(
        "DELETE FROM keys 
         WHERE user_id = $1",
        auth.0
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;
    
    sqlx::query!(
        "DELETE FROM users 
         WHERE id = $1",
        auth.0
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;
    
    tx.commit()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let email_body = format!(
        "We're sorry to see you go. Your account has been successfully deleted.\n\
        - Time: {}\n\n\
        Important Security Notes:\n\
        - All your keys have been permanently deleted\n\
        - Your recovery codes have been invalidated\n\
        - Your personal data has been removed from our systems\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let email_request = EmailRequest {
        sender: GLOBAL_SENDER_EMAIL.to_string(),
        recipient: record.email,
        subject: "Account Successfully Deleted👋".to_string(),
        body: email_body,
    };

    enqueue_email(email_request).await;

    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_current_user,
        change_user,
        delete_user,
    ]
}
