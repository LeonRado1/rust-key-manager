use rocket::serde::json::Json;
use serde::Serialize;

/// https://github.com/block/goose/blob/main/crates/goose-server/src/routes/health.rs

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
}

/// Simple status endpoint that returns 200 OK when the server is running
#[get("/")]
async fn status() -> Json<StatusResponse> {
    Json(StatusResponse { status: "ok" })
}

/// Health check route
pub fn routes() -> Vec<rocket::Route> {
    routes![status]
}

