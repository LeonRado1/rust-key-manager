use rocket::serde::json::Json;
use rocket::http::Status;
use sqlx::PgPool;
use rocket::State;
use crate::middleware::LoggedUser;
use crate::models::{Key, PartialKey, KeyRequest, PartialKeyRequest, KeysResponse};
use crate::services::{encrypt, generate_ssh_key_pair};
use crate::utils::constants::SSH_KEY;

#[get("/")]
async fn get_keys(
    pool: &State<PgPool>,
    auth: LoggedUser
) -> Result<Json<Vec<PartialKey>>, Status> {
    let keys = sqlx::query_as!(
        PartialKey,
        "SELECT keys.id, key_name, key_description, key_type_id, key_type, key_tag, expiration_date
         FROM keys 
         JOIN key_types 
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1",
        auth.0
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;
    
    Ok(Json(keys))
}

#[post("/", data = "<request_data>")]
async fn create_key(
    pool: &State<PgPool>,
    request_data: Json<KeyRequest>,
    auth: LoggedUser
) -> Result<(), Status> {
    
    let result = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::Unauthorized })?;
    
    let password_hash: String = result.password_hash;
    
    if request_data.key_type_id == SSH_KEY {
        let ssh_key_pair = generate_ssh_key_pair();
        
        let (private_key, public_key) = match ssh_key_pair {
            Ok(ssh_key_pair) => (ssh_key_pair.private_key, ssh_key_pair.public_key),
            Err(e) => return Err(e)
        };
        
        let encrypted_data = encrypt(&private_key, &password_hash);

        match encrypted_data {
            Ok(encrypted_data) => {
                sqlx::query!(
                    "INSERT INTO keys (
                        user_id, key_name, key_value, key_description, key_type_id, key_tag, key_pair_value, expiration_date, salt, nonce
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    auth.0, 
                    request_data.key_name, 
                    encrypted_data.ciphertext, 
                    request_data.key_description, 
                    request_data.key_type_id,
                    request_data.key_tag,
                    public_key,
                    request_data.expiration_date, 
                    encrypted_data.salt,
                    encrypted_data.nonce
                )
                .execute(pool.inner())
                .await
                .map_err(|_e| Status::InternalServerError)?;

                Ok(())
            }
            Err(e) => Err(e)
        }
    }
    else {
        let encrypted_data = encrypt(&(request_data.key_value), &password_hash);
        
        match encrypted_data {
            Ok(encrypted_data) => {
                sqlx::query!(
                    "INSERT INTO keys (
                        user_id, key_name, key_value, key_description, key_type_id, key_tag, expiration_date, salt, nonce
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    auth.0, 
                    request_data.key_name, 
                    encrypted_data.ciphertext, 
                    request_data.key_description, 
                    request_data.key_type_id,
                    request_data.key_tag,
                    request_data.expiration_date, 
                    encrypted_data.salt,
                    encrypted_data.nonce
                )
                .execute(pool.inner())
                .await
                .map_err(|_e| Status::InternalServerError)?;
                
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}

/// Set expiration date for a specific key
/// # Path Parameters:
/// - `key_id`: id of the key
///
/// # Body
/// - `user_id`: id of the user
/// - `expiration_date`: expiration date of the key
#[patch("/<key_id>", data = "<request_data>")]
async fn set_expiration(
    pool: &State<PgPool>,
    key_id: i32,
    request_data: Json<PartialKeyRequest>,
    auth: LoggedUser
) -> Result<Json<KeysResponse>, Status> {
    if let Some(expiration_date) = request_data.expiration_date {
        let updated = sqlx::query(
            "UPDATE keys SET expiration_date = $1 WHERE id = $2 AND user_id = $3"
        )
            .bind(expiration_date)
            .bind(key_id)
            .bind(auth.0)
        .execute(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            Status::InternalServerError
        })?;

        if updated.rows_affected() == 0 { return Err(Status::NotFound); }

        return Ok(Json(KeysResponse {
            message: "Key expiration date updated successfully".to_string(),
        }));
    }

    Err(Status::UnprocessableEntity)
}

/// Delete a key
/// # Path Parameters:
/// - `key_id`: id of the key
#[delete("/<key_id>")]
async fn delete(
    pool: &State<PgPool>,
    key_id: i32,
    auth: LoggedUser
) -> Result<Json<KeysResponse>, Status> {
    let deleted = sqlx::query(
        "DELETE FROM keys WHERE id = $1 AND user_id = $2"
    )
        .bind(key_id)
        .bind(auth.0)
    .execute(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if deleted.rows_affected() == 0 { return Err(Status::NotFound); }

    Ok(Json(KeysResponse {
        message: "Key deleted successfully".to_string()
    }))
}

/// Getting the list of expired keys for a specific user
#[get("/expired")]
async fn expired(
    pool: &State<PgPool>,
    auth: LoggedUser
) -> Result<Json<Vec<Key>>, Status> {
    let expired_keys = sqlx::query_as::<_, Key>(
        "SELECT id, key_value, key_name, key_type, key_description, expiration_date, created_at
         FROM keys
         WHERE user_id = $1 AND expiration_date < NOW()"
    )
    .bind(auth.0)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if expired_keys.is_empty() {
        // Keys not found, or expiration date is not set
        return Err(Status::NotFound);
    }

    Ok(Json(expired_keys))
}

pub fn routes() -> Vec<rocket::Route> {
    // TODO: add patch method if needed, create a new method for generating or encrypting keys
    // TODO: add return logic
    routes![
        get_keys, expired, // Get keys
        create_key, delete, // Creation or deletion
        set_expiration, // Update key properties
         // Import keys(Post)
    ]
}

