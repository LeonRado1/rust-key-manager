use std::env;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use once_cell::sync::Lazy;
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

static REQUEST_ATTEMPTS: Lazy<DashMap<String, (u32, Instant)>> = Lazy::new(DashMap::new);

const MAX_REQUEST_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(300);

pub struct RequestLimitGuard(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestLimitGuard {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ip = request.client_ip().map(|ip| ip.to_string()).unwrap_or("unknown".into());

        if let Some((count, last)) = REQUEST_ATTEMPTS.get(&ip).map(|e| *e.value()) {
            if count >= MAX_REQUEST_ATTEMPTS && last.elapsed() < LOCKOUT_DURATION {
                return Outcome::Error((Status::TooManyRequests, "Too many requests from this IP address"));
            }
        }

        Outcome::Success(RequestLimitGuard(ip))
    }
}

pub fn record_failed_attempt(ip_address: &str) {
    let now = Instant::now();
    REQUEST_ATTEMPTS.entry(ip_address.to_string())
        .and_modify(|e| {
            e.0 += 1;
            e.1 = now;
        })
        .or_insert((1, now));
}

pub fn reset_limit_attempts(ip_address: &str) {
    REQUEST_ATTEMPTS.remove(ip_address);
}
