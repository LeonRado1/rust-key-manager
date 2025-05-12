use reqwest::StatusCode;

pub fn get_error_message_from_code(status: StatusCode) -> Option<String>
{
    if status.is_success() {
        return None;       
    }

    let message = match status {
        StatusCode::UNAUTHORIZED => "Credentials you provided are wrong, or your session is expired.",
        StatusCode::FORBIDDEN => "You don't have permission to access this resource.",
        StatusCode::NOT_FOUND => "The requested resource was not found.",
        StatusCode::BAD_REQUEST => "Invalid request data provided.",
        StatusCode::INTERNAL_SERVER_ERROR => "An internal server error occurred. Please try again later.",
        StatusCode::CONFLICT => "A user with the same email or username already exists.",
        StatusCode::UNPROCESSABLE_ENTITY => "The password is too short or not enough complex.",
        StatusCode::EXPECTATION_FAILED => "Values you provided are not in correct format and need to be changed.",
        StatusCode::TOO_MANY_REQUESTS => "You have exceeded the rate limit. Please try again after 5 minutes.",
        _ => "An unexpected error occurred. Please try again later.",
    };

    Some(message.to_string())
}
