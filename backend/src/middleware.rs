use std::env;
use rocket::http::Status;
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use crate::utils::jwt_token::validate_jwt_token;

pub struct LoggedUser(pub i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedUser {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => return Outcome::Error((Status::InternalServerError, "JWT_SECRET is not set"))
        };

        let auth_header = request.headers().get_one("Authorization");
        
        let access_token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            Some(_) => return Outcome::Error((Status::BadRequest, "Invalid Authorization header format")),
            _ => return Outcome::Error((Status::Unauthorized, "Authorization header not found")),
        };

        match validate_jwt_token(access_token, &jwt_secret) {
            Ok(claims) => {
                let user_id = match claims.sub.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
                };

                Outcome::Success(LoggedUser(user_id))
            }
            Err(_) => Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
        }
    }
}
