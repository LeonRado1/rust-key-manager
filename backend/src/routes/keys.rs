use rocket::serde::json::Json;
use rocket::http::Status;
use sqlx::PgPool;
use rocket::State;

use crate::models::{Key, KeyRequest, KeysResponse};
use crate::routes::auth::LoggedUser;


/// Get a list of keys for a specific user
#[get("/<user_id>")]
async fn list(
    pool: &State<PgPool>,
    user_id: i32,
    auth: LoggedUser
) -> Result<Json<Vec<Key>>, Status> {
    // Control access to the only user's keys
    if auth.0 != user_id { return Err(Status::Forbidden); }

    let keys = sqlx::query_as::<_, Key>(
        "SELECT id, key_value, key_name, key_type, key_description, created_at
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

#[post("/", data = "<request_data>")]
async fn create(
    pool: &State<PgPool>,
    request_data: Json<KeyRequest>,
    auth: LoggedUser
) -> Result<Json<KeysResponse>, Status> {
    if auth.0 != request_data.user_id { return Err(Status::Forbidden); }

    sqlx::query_as::<_, Key>(
        "INSERT INTO keys (
                  user_id,
                  key_value,
                  key_name,
                  key_type,
                  key_description
                  ) VALUES ($1, $2, $3, $4, $5)
                  RETURNING id, key_value, key_name, key_type, key_description, created_at"
    )
    .bind(request_data.user_id)
    .bind(&request_data.key_value)
    .bind(&request_data.key_name)
    .bind(&request_data.key_type)
    .bind(&request_data.key_description)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| {
        Status::InternalServerError
    })?;

    Ok(Json(KeysResponse {
        message: "Key saved successfully".to_string()
    }))
}

// #[put("/<key_id>", data = "<request_data>")]
// async fn update(key_id: String, request_data: Json<KeyRequest>) -> Json<KeysResponse> {
//
// }

// #[patch("/<key_id>", data = "<request_data>")]
// updating key values as: name, key, description...

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

pub fn routes() -> Vec<rocket::Route> {
    routes![list, create, delete]
}

