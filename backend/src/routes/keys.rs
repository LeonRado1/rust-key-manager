use chrono::{Local, NaiveDateTime};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::http::{Header, Status};
use sqlx::PgPool;
use rocket::{Response, State};
use tokio::io::AsyncReadExt;
use crate::middleware::LoggedUser;
use crate::models::{Key, PartialKey, KeyRequest, ImportKeyForm, UpdateKeyRequest};
use crate::services::{decrypt, encrypt, generate_ssh_key_pair, generate_token, KeyEncoding, KeyType};
use crate::utils::constants::{PASSWORD, SSH_KEY, TOKEN};
use crate::utils::validation::{validate_openssh_key, validate_openssh_private_key, validate_password};

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
         WHERE user_id = $1
           AND (expiration_date IS NULL OR expiration_date > $2)
           AND is_revoked = false
         ORDER BY COALESCE(updated_at, created_at) DESC;",
        auth.0,
        Local::now().naive_local()
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    Ok(Json(keys))
}

#[get("/key/<key_id>")]
async fn get_key_detail(
    pool: &State<PgPool>,
    key_id: i32,
    auth: LoggedUser
) -> Result<Json<Key>, Status> {
    let mut key = sqlx::query_as!(
        Key,
        "SELECT keys.id, key_name, key_value, key_description, key_type_id, key_type, key_tag,
            key_pair_value, expiration_date, created_at, updated_at, is_revoked
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::NotFound })?;

    let record = sqlx::query!(
        "SELECT password_hash, salt, nonce
         FROM keys
         JOIN users ON users.id = keys.user_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    key.key_value = decrypt(&key.key_value, &record.salt, &record.nonce, &record.password_hash)?;

    Ok(Json(key))
}

#[get("/revoked")]
async fn get_revoked_keys(
    pool: &State<PgPool>,
    auth: LoggedUser
) -> Result<Json<Vec<PartialKey>>, Status> {
    let keys = sqlx::query_as!(
        PartialKey,
        "SELECT keys.id, key_name, key_description, key_type_id, key_type, key_tag, expiration_date
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND is_revoked = true
         ORDER BY COALESCE(updated_at, created_at) DESC;",
        auth.0
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    Ok(Json(keys))
}

#[get("/expired")]
async fn get_expired_keys(
    pool: &State<PgPool>,
    auth: LoggedUser
) -> Result<Json<Vec<PartialKey>>, Status> {
    let keys = sqlx::query_as!(
        PartialKey,
        "SELECT keys.id, key_name, key_description, key_type_id, key_type, key_tag, expiration_date
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND expiration_date < $2 AND is_revoked = false
         ORDER BY COALESCE(updated_at, created_at) DESC;",
        auth.0,
        Local::now().naive_local()
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    Ok(Json(keys))
}

#[post("/", data = "<request_data>")]
async fn create_key(
    pool: &State<PgPool>,
    mut request_data: Json<KeyRequest>,
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
                        user_id, key_name, key_value, key_description, key_type_id, key_tag, key_pair_value, salt, nonce
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    auth.0,
                    request_data.key_name, 
                    encrypted_data.ciphertext, 
                    request_data.key_description, 
                    request_data.key_type_id,
                    request_data.key_tag,
                    public_key,
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

        if request_data.key_type_id == PASSWORD {
            if request_data.key_value.is_empty() {
                return Err(Status::BadRequest);
            }

            if let Err(_e) = validate_password(&request_data.key_value) {
                return Err(Status::UnprocessableEntity);
            };
        };

        if request_data.key_value.is_empty() {
            let (key_type, encoding) = if request_data.key_type_id == TOKEN {
                (KeyType::Token, KeyEncoding::Base64)
            } else {
                (KeyType::ApiKey, KeyEncoding::Hex)
            };

            request_data.key_value = generate_token(key_type, encoding)?;
        }

        let encrypted_data = encrypt(&(request_data.key_value), &password_hash);

        let expiration_date: Option<NaiveDateTime> = if request_data.expiration_date.is_none() {
            None
        } else {
            let date_str = request_data.expiration_date.as_ref().unwrap();

            let date = NaiveDateTime::parse_from_str(&date_str, "%y/%m/%d %H:%M:%S")
                .map_err(|_| Status::ExpectationFailed)?;

            if date < Local::now().naive_local() {
                return Err(Status::ExpectationFailed);
            }

            Some(date)
        };

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
                    expiration_date,
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

#[post("/import", data = "<form>")]
async fn import_ssh_key(
    pool: &State<PgPool>,
    form: Form<ImportKeyForm<'_>>,
    auth: LoggedUser
) -> Result<(), Status> {

    let request_data: KeyRequest = serde_json::from_str(&form.json)
        .map_err(|_| Status::BadRequest)?;

    let mut content = String::new();
    form.file.open().await
        .map_err(|_| Status::InternalServerError)?
        .read_to_string(&mut content)
        .await
        .map_err(|_| Status::BadRequest)?;

    if !validate_openssh_private_key(&content) || !validate_openssh_key(request_data.key_value.as_str()) {
        return Err(Status::ExpectationFailed);
    }

    let result = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::Unauthorized)?;

    let password_hash: String = result.password_hash;

    let encrypted_data = encrypt(&content, &password_hash)
        .map_err(|_| Status::InternalServerError)?;

    sqlx::query!(
        "INSERT INTO keys (
            user_id, key_name, key_value, key_description, key_type_id, key_tag, key_pair_value, salt, nonce
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        auth.0,
        request_data.key_name,
        encrypted_data.ciphertext,
        request_data.key_description,
        request_data.key_type_id,
        request_data.key_tag,
        request_data.key_value,
        encrypted_data.salt,
        encrypted_data.nonce
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(())
}

#[patch("/rotate/<key_id>")]
async fn rotate_key(
    pool: &State<PgPool>,
    key_id: i32,
    auth: LoggedUser
) -> Result<Json<Key>, Status> {
    let mut key = sqlx::query_as!(
        Key,
        "SELECT keys.id, key_name, key_value, key_description, key_type_id, key_type, key_tag,
            key_pair_value, expiration_date, created_at, updated_at, is_revoked
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::NotFound })?;

    if key.key_type_id == PASSWORD {
        return Err(Status::ExpectationFailed);
    }

    let record = sqlx::query!(
        "SELECT password_hash
         FROM users
         WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    if key.key_type_id == SSH_KEY {
        let ssh_key_pair = generate_ssh_key_pair();

        let (private_key, public_key) = match ssh_key_pair {
            Ok(ssh_key_pair) => (ssh_key_pair.private_key, ssh_key_pair.public_key),
            Err(e) => return Err(e)
        };

        let encrypted_data = encrypt(&private_key, &record.password_hash)?;

        let date = Local::now().naive_utc();

        sqlx::query!(
            "UPDATE keys
             SET key_value = $1, key_pair_value = $2, salt = $3, nonce = $4, updated_at = $5
             WHERE id = $6",
            encrypted_data.ciphertext,
            public_key,
            encrypted_data.salt,
            encrypted_data.nonce,
            &date,
            key.id
        )
        .execute(pool.inner())
        .await
        .map_err(|_e| Status::InternalServerError)?;

        key.key_value = private_key;
        key.key_pair_value = Some(public_key);
        key.updated_at = Some(date);
    }
    else {
        let (key_type, encoding) = if key.key_type_id == TOKEN {
            (KeyType::Token, KeyEncoding::Base64)
        } else {
            (KeyType::ApiKey, KeyEncoding::Hex)
        };

        let generated = generate_token(key_type, encoding)?;

        let encrypted_data = encrypt(&generated, &record.password_hash)?;

        let date = Local::now().naive_utc();

        sqlx::query!(
            "UPDATE keys
             SET key_value = $1, salt = $2, nonce = $3, updated_at = $4
             WHERE id = $5",
            encrypted_data.ciphertext,
            encrypted_data.salt,
            encrypted_data.nonce,
            &date,
            key.id
        )
        .execute(pool.inner())
        .await
        .map_err(|_e| Status::InternalServerError)?;

        key.key_value = generated;
        key.updated_at = Some(date);
    }

    Ok(Json(key))
}

#[patch("/<key_id>")]
async fn revoke_key(
    pool: &State<PgPool>,
    key_id: i32,
    auth: LoggedUser
) -> Result<(), Status> {
    let exists = sqlx::query!(
        "SELECT id FROM keys WHERE id = $1 AND user_id = $2",
        key_id,
        auth.0
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if exists.is_none() {
        return Err(Status::Forbidden);
    }

    sqlx::query!(
        "UPDATE keys
         SET is_revoked = true, updated_at = $1
         WHERE id = $2",
        Local::now().naive_utc(),
        key_id
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(())
}

#[put("/change", data = "<request_data>")]
async fn change_key(
    pool: &State<PgPool>,
    request_data: Json<UpdateKeyRequest>,
    auth: LoggedUser
) -> Result<Json<Key>, Status> {
    let mut key = sqlx::query_as!(
        Key,
        "SELECT keys.id, key_name, key_value, key_description, key_type_id, key_type, key_tag,
            key_pair_value, expiration_date, created_at, updated_at, is_revoked
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        request_data.key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::NotFound })?;

    if key.key_type_id != PASSWORD {
        return Err(Status::BadRequest);
    }

    let record = sqlx::query!(
        "SELECT password_hash
         FROM users
         WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    let new_password = match &request_data.new_password {
        Some(pwd) => pwd,
        None => return Err(Status::BadRequest)
    };

    if let Err(_) = validate_password(new_password) {
        return Err(Status::UnprocessableEntity);
    }

    let encrypted_data = encrypt(new_password, &record.password_hash)?;

    let date = Local::now().naive_utc();

    sqlx::query!(
        "UPDATE keys
         SET key_value = $1, salt = $2, nonce = $3, updated_at = $4
         WHERE id = $5",
        encrypted_data.ciphertext,
        encrypted_data.salt,
        encrypted_data.nonce,
        &date,
        key.id
    )
    .execute(pool.inner())
    .await
    .map_err(|_e| Status::InternalServerError)?;

    key.key_value = new_password.to_string();
    key.updated_at = Some(date);

    Ok(Json(key))
}

#[put("/extend", data = "<request_data>")]
async fn extend_key(
    pool: &State<PgPool>,
    request_data: Json<UpdateKeyRequest>,
    auth: LoggedUser
) -> Result<Json<Key>, Status> {
    let mut key = sqlx::query_as!(
        Key,
        "SELECT keys.id, key_name, key_value, key_description, key_type_id, key_type, key_tag,
            key_pair_value, expiration_date, created_at, updated_at, is_revoked
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        request_data.key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::NotFound })?;

    if key.key_type_id != TOKEN {
        return Err(Status::BadRequest);
    }

    let new_expiration_date = match &request_data.new_expiration_date {
        Some(date_str) => {
            let date = NaiveDateTime::parse_from_str(date_str, "%y/%m/%d %H:%M:%S")
                .map_err(|_| Status::ExpectationFailed)?;

            if date < Local::now().naive_local() {
                return Err(Status::ExpectationFailed);
            }

            date
        },
        None => return Err(Status::BadRequest)
    };

    let date = Local::now().naive_utc();

    sqlx::query!(
        "UPDATE keys
         SET expiration_date = $1, updated_at = $2
         WHERE id = $3",
        new_expiration_date,
        &date,
        key.id
    )
    .execute(pool.inner())
    .await
    .map_err(|_e| Status::InternalServerError)?;

    key.expiration_date = Some(new_expiration_date);
    key.updated_at = Some(date);

    Ok(Json(key))
}

#[put("/extend/rotate", data = "<request_data>")]
async fn extend_rotate_key(
    pool: &State<PgPool>,
    request_data: Json<UpdateKeyRequest>,
    auth: LoggedUser
) -> Result<Json<Key>, Status> {
    let mut key = sqlx::query_as!(
        Key,
        "SELECT keys.id, key_name, key_value, key_description, key_type_id, key_type, key_tag,
            key_pair_value, expiration_date, created_at, updated_at, is_revoked
         FROM keys
         JOIN key_types
            ON key_types.id = keys.key_type_id
         WHERE user_id = $1 AND keys.id = $2",
        auth.0,
        request_data.key_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::NotFound })?;

    if key.key_type_id != TOKEN {
        return Err(Status::BadRequest);
    }

    let new_expiration_date = match &request_data.new_expiration_date {
        Some(date_str) => {
            let date = NaiveDateTime::parse_from_str(date_str, "%y/%m/%d %H:%M:%S")
                .map_err(|_| Status::ExpectationFailed)?;

            if date < Local::now().naive_local() {
                return Err(Status::ExpectationFailed);
            }

            date
        },
        None => return Err(Status::BadRequest)
    };

    let record = sqlx::query!(
        "SELECT password_hash
         FROM users
         WHERE id = $1",
        auth.0
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_e| { Status::InternalServerError })?;

    let generated = generate_token(KeyType::Token, KeyEncoding::Base64)?;

    let encrypted_data = encrypt(&generated, &record.password_hash)?;

    let date = Local::now().naive_utc();

    sqlx::query!(
        "UPDATE keys
         SET key_value = $1, salt = $2, nonce = $3, expiration_date = $4, updated_at = $5
         WHERE id = $6",
        encrypted_data.ciphertext,
        encrypted_data.salt,
        encrypted_data.nonce,
        new_expiration_date,
        date,
        key.id
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    key.key_value = generated;
    key.expiration_date = Some(new_expiration_date);
    key.updated_at = Some(date);

    Ok(Json(key))
}

#[delete("/<key_id>")]
async fn delete_key(
    pool: &State<PgPool>,
    key_id: i32,
    auth: LoggedUser
) -> Result<(), Status> {
    let exists = sqlx::query!(
        "SELECT id FROM keys WHERE id = $1 AND user_id = $2",
        key_id,
        auth.0
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if exists.is_none() {
        return Err(Status::Forbidden);
    }

    sqlx::query!(
        "DELETE FROM keys
         WHERE id = $1",
        key_id
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_keys, get_key_detail, get_revoked_keys, get_expired_keys,
        create_key, import_ssh_key,
        change_key, extend_key, extend_rotate_key, rotate_key,
        revoke_key, delete_key
    ]
}
