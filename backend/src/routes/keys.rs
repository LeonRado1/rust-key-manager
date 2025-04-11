use rocket::serde::json::Json;
use rocket::http::Status;
use sqlx::PgPool;
use rocket::State;

use crate::models::{Key, KeyRequest, PartialKeyRequest, KeysResponse};
use crate::routes::auth::LoggedUser;


/// Get a list of keys for a specific user
/// # Path Parameters:
/// - `user_id`: id of the user
#[get("/<user_id>")]
async fn list(
    pool: &State<PgPool>,
    user_id: i32,
    auth: LoggedUser
) -> Result<Json<Vec<Key>>, Status> {
    // Control access to the only user's keys
    if auth.0 != user_id { return Err(Status::Forbidden); }

    let keys = sqlx::query_as::<_, Key>(
        "SELECT id, key_value, key_name, key_type, key_description, expiration_date, created_at
         FROM keys WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        Status::InternalServerError
    })?;

    if keys.is_empty() {
        eprintln!("No keys found");
        return Err(Status::NotFound);
    }

    Ok(Json(keys))
}

/// Create a new key
/// Body:
/// - `user_id`: id of the user
/// - `key_value`: value of the key
/// - `key_name`: name of the key
/// - `key_type`: type of the key
/// - Optional fields:
/// - `key_description`: description of the key
/// - `expiration_date`: expiration date of the key
#[post("/", data = "<request_data>")]
async fn create(
    pool: &State<PgPool>,
    request_data: Json<KeyRequest>,
    auth: LoggedUser
) -> Result<Json<KeysResponse>, Status> {
    if auth.0 != request_data.user_id { return Err(Status::Forbidden); }

    let key = sqlx::query_as::<_, Key>(
        "INSERT INTO keys (
                  user_id, key_value, key_name, key_type, key_description, expiration_date
                  ) VALUES ($1, $2, $3, $4, $5, $6)
                  RETURNING id, key_value, key_name, key_type, key_description, expiration_date, created_at"
    )
    .bind(request_data.user_id)
    .bind(&request_data.key_value)
    .bind(&request_data.key_name)
    .bind(&request_data.key_type)
    .bind(&request_data.key_description)
    .bind(&request_data.expiration_date)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| {
        Status::InternalServerError
    })?;

    Ok(Json(KeysResponse {
        message: format!("Key {} saved successfully", key.id)
    }))
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
    // Control existence of the user id in the request
    if let Some(user_id) = request_data.user_id {
        // Control access to the only user's keys
        if auth.0 != user_id {
            return Err(Status::Forbidden);
        }
    } else {
        return Err(Status::UnprocessableEntity);
    }

    if let Some(expiration_date) = request_data.expiration_date {
        let updated = sqlx::query!(
            "UPDATE keys SET expiration_date = $1 WHERE id = $2",
            expiration_date,
            key_id
        )
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
    // Control access to the only user's keys
    if auth.0 != key_id { return Err(Status::Forbidden); }

    let deleted = sqlx::query!(
        "DELETE FROM keys WHERE id = $1",
        key_id
    )
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

/// Import keys from a JSON file
/// # Body
/// - `user_id`: id of the user
/// - `key_value`: value of the key
/// - `key_name`: name of the key
/// - `key_type`: type of the key
/// - Optional fields:
/// - `key_description`: description of the key
/// - `expiration_date`: expiration date of the key
#[post("/import", data = "<request_data>")]
async fn import(
    pool: &State<PgPool>,
    request_data: Json<Vec<KeyRequest>>,
    auth: LoggedUser
) -> Result<Json<KeysResponse>, Status> {
    // Control access to the only user's keys
    if request_data.iter().any(|key| auth.0 != key.user_id) { return Err(Status::Forbidden); }

    for key in request_data.iter() {
        sqlx::query(
            "INSERT INTO keys (
                      user_id,
                      key_value,
                      key_name,
                      key_type,
                      key_description,
                      expiration_date
                      ) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(key.user_id)
        .bind(&key.key_value)
        .bind(&key.key_name)
        .bind(&key.key_type)
        .bind(&key.key_description)
        .bind(&key.expiration_date)
        .execute(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            Status::InternalServerError
        })?;
    }

    Ok(Json(KeysResponse {
        message: "Keys imported successfully".to_string()
    }))
}


pub fn routes() -> Vec<rocket::Route> {
    // TODO: add patch method if needed, create a new method for generating or encrypting keys
    // TODO: add return logic
    routes![
        list, expired, // Get keys
        create, delete, // Creation or deletion
        set_expiration, // Update key properties
        import // Import keys(Post)
    ]
}

