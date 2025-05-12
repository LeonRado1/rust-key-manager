use sqlx::PgPool;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::State;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Local;
use rand::Rng;
use rand::rngs::OsRng;
use crate::middleware::{record_failed_attempt, reset_limit_attempts, RequestLimitGuard};
use crate::models::*;
use crate::services::{decrypt, encrypt, enqueue_email, EmailRequest, GLOBAL_SENDER_EMAIL};
use crate::utils::jwt_token::generate_jwt_token;
use crate::utils::validation::{validate_email, validate_password, validate_username};

#[post("/login", data = "<request_data>")]
async fn login(
    pool: &State<PgPool>,
    request_data: Json<LoginRequest>,
    ip: RequestLimitGuard
) -> Result<Json<AuthResponse>, Status> {
    let record = sqlx::query!(
        "SELECT id, username, password_hash, email FROM users WHERE email = $1",
        &request_data.email
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::Unauthorized)?;

    let is_valid_password = verify(&request_data.password, &record.password_hash)
        .map_err(|_| Status::InternalServerError)?;
    
    if !is_valid_password {
        record_failed_attempt(&ip.0);
        return Err(Status::Unauthorized); 
    }
    
    reset_limit_attempts(&ip.0);

    let token = generate_jwt_token(record.id)
        .map_err(|_| Status::InternalServerError)?;

    let user = User {
        id: record.id,
        username: record.username,
        email: record.email,
    };
    
    Ok(Json(AuthResponse { user, token }))
}

#[post("/register", data = "<request_data>")]
async fn register(
    pool: &State<PgPool>,
    request_data: Json<RegisterRequest>
) -> Result<Json<AuthResponse>, Status> {
    
    if let Err(_e) = validate_password(&request_data.password) {
        return Err(Status::UnprocessableEntity);
    };

    if let Err(_e) = validate_username(&request_data.username) {
        return Err(Status::ExpectationFailed);
    };

    if let Err(_e) = validate_email(&request_data.email) {
        return Err(Status::ExpectationFailed);
    };

    let password_hash = hash(&request_data.password, DEFAULT_COST)
        .map_err(|_| { Status::InternalServerError })?;

    let record = sqlx::query!(
        "INSERT INTO users (username, email, password_hash)
         VALUES ($1, $2, $3)
         RETURNING id, username, email",
        &request_data.username,
        &request_data.email,
        &password_hash
    )
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

    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = OsRng;

    let mut recovery_codes = Vec::new();

    for _ in 0..3 {
        let recovery_code: String = (0..12)
            .map(|_| {
                let idx = rng.gen_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect();

        sqlx::query!(
            "INSERT INTO recovery_codes (user_id, code)
             VALUES ($1, $2)",
            record.id,
            recovery_code
        )
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

        recovery_codes.push(recovery_code);
    }

    let email_body = format!(
        "Welcome to Key Manager, {}!\n\
        Your account has been successfully created. Here are your recovery codes.\
        Please store them safely as they will be needed if you ever need to reset your password:\n\n\
        {}\n\n\
        Important:\n\
        - Each code can only be used once\n\
        - Store these codes in a secure location\n\
        - Don't share these codes with anyone\n",
        record.username,
        recovery_codes.join("\n")
    );

    let email_request = EmailRequest {
        sender: GLOBAL_SENDER_EMAIL.to_string(),
        recipient: record.email.clone(),
        subject: "Welcome to Key Manager🔐".to_string(),
        body: email_body,
    };

    enqueue_email(email_request).await;

    let token = generate_jwt_token(record.id)
        .map_err(|_| Status::InternalServerError)?;

    let user = User {
        id: record.id,
        username: record.username,
        email: record.email,
    };

    Ok(Json(AuthResponse { user, token }))
}

#[put("/change-password", data = "<request_data>")]
async fn change_password(
    pool: &State<PgPool>,
    request_data: Json<ChangePasswordRequest>,
    ip: RequestLimitGuard
) -> Result<(), Status> {
    let user = sqlx::query!(
        "SELECT id, username, password_hash, email FROM users WHERE email = $1",
        &request_data.email
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::Unauthorized)?;
    
    let code = sqlx::query!(
        "SELECT id FROM recovery_codes 
         WHERE user_id = $1 AND code = $2 AND is_used = false",
        user.id,
        &request_data.recovery_code
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;
    
    if code.is_none() {
        record_failed_attempt(&ip.0);
        return Err(Status::Unauthorized);
    }
    
    reset_limit_attempts(&ip.0);
    let code = code.unwrap();

    if let Err(_e) = validate_password(&request_data.new_password) {
        return Err(Status::UnprocessableEntity);
    }
    
    let new_password_hash = hash(&request_data.new_password, DEFAULT_COST)
        .map_err(|_| Status::InternalServerError)?;
    
    let old_password_hash = user.password_hash;

    let keys = sqlx::query!(
        "SELECT id, key_value, salt, nonce 
         FROM keys WHERE user_id = $1",
        user.id
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let mut tx = pool.inner()
        .begin()
        .await
        .map_err(|_| Status::InternalServerError)?;

    for key in keys {
        let decrypted_value = decrypt(&key.key_value, &key.salt, &key.nonce, &old_password_hash)?;
        let encrypted_value = encrypt(&decrypted_value, &new_password_hash)?;
        
        sqlx::query!(
            "UPDATE keys 
             SET key_value = $1, salt = $2, nonce = $3
             WHERE id = $4",
            encrypted_value.ciphertext,
            encrypted_value.salt,
            encrypted_value.nonce,
            key.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| Status::InternalServerError)?;
    }
    
    sqlx::query!(
        "UPDATE users 
         SET password_hash = $1
         WHERE id = $2",
        new_password_hash,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    sqlx::query!(
        "UPDATE recovery_codes  
         SET is_used = true
         WHERE id = $1",
        code.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    tx.commit()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let email_body = format!(
        "Your account password was recently changed using a recovery code.\n\
        - Time: {}\n\
        - Recovery Code Used: {}\n\n\
        Important:\n\
        If you did not initiate this password change, please contact our support team.\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        &request_data.recovery_code
    );

    let email_request = EmailRequest {
        sender: GLOBAL_SENDER_EMAIL.to_string(),
        recipient: user.email,
        subject: "Password Change Alert🚨".to_string(),
        body: email_body,
    };

    enqueue_email(email_request).await;

    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        login,
        register,
        change_password,
    ]
}
