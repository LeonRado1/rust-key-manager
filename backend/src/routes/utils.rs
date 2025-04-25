use rocket::serde::{json::Json};
use rocket::http::Status;
use rocket::{State};
use sqlx::PgPool;

use crate::models::{
    GeneratePasswordRequest,
    GeneratePasswordResponse
};
use crate::routes::auth::LoggedUser;
use crate::routes::users::check_user_exists;

use crate::services::generate_password;


#[post("/password/generate", data = "<request_data>")]
async fn password(
    pool: &State<PgPool>,
    request_data: Json<GeneratePasswordRequest>,
    auth: LoggedUser
) -> Result<Json<GeneratePasswordResponse>, Status> {
    // Check user authentication
    check_user_exists(pool.inner(), auth.0).await?;

    if request_data.length < 8 {
        return Err(Status::BadRequest);
    } else if request_data.length > 256 {
        return Err(Status::BadRequest);
    }

    let password = generate_password(
        request_data.length,
        request_data.include_special_symbols,
        request_data.include_numbers,
        request_data.include_uppercase,
        request_data.include_lowercase
    );

    Ok(Json(GeneratePasswordResponse { password }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![password]
}

