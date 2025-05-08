use reqwest::Client;
use crate::helpers::error_codes::get_error_message_from_code;
use crate::models::user::{RegisterRequest, LoginRequest, AuthResponse, ChangeUserRequest, User};

pub async fn register(data: RegisterRequest) -> Result<AuthResponse, String> {
    let client = Client::new();
    let response = client.post("http://127.0.0.1:8000/auth/register")
        .json(&data)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;
    
    Ok(response)
}

pub async fn login(data: LoginRequest) -> Result<AuthResponse, String> { 
    let client = Client::new();
    let response = client.post("http://127.0.0.1:8000/auth/login")
        .json(&data)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;
    
    Ok(response)
}

pub async fn get_current_user(token: &str) -> Result<AuthResponse, String> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/users")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;
    
    Ok(response)
}

pub async fn change_user_details(
    token: &str, 
    data: ChangeUserRequest
) -> Result<User, String> {
    let client = Client::new();
    let response = client.put("http://127.0.0.1:8000/users")
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn delete_user(
    token: &str,
    data: ChangeUserRequest
) -> Result<(), String> {
    let client = Client::new();
    let response = client.delete("http://127.0.0.1:8000/users")
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }
    
    Ok(())
}
