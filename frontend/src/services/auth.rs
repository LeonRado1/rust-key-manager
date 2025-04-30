use reqwest::Client;
use crate::models::user::{RegisterRequest, LoginRequest, AuthResponse};

pub async fn register(data: RegisterRequest) -> Result<AuthResponse, reqwest::Error> {
    let client = Client::new();
    let response = client.post("http://127.0.0.1:8000/auth/register")
        .json(&data)
        .send()
        .await?;
    
    let response = response.json().await?;
    Ok(response)
}

pub async fn login(data: LoginRequest) -> Result<AuthResponse, reqwest::Error> { 
    let client = Client::new();
    let response = client.post("http://127.0.0.1:8000/auth/login")
        .json(&data)
        .send()
        .await?;

    let response = response.json().await?;
    Ok(response)
}

pub async fn get_current_user(token: &str) -> Result<AuthResponse, reqwest::Error> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/auth/currentUser")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let response = response.json().await?;
    Ok(response)
}
