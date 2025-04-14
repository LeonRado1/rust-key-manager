use reqwest::Client;
use crate::models::user::{SignupRequest, LoginRequest, AuthResponse};

pub async fn signup(data: SignupRequest) -> Result<AuthResponse, reqwest::Error> {
    let client = Client::new();
    let response = client.post("api/signup")
        .json(&data)
        .send()
        .await?;

    let response = response.json().await?;
    Ok(response)
}

pub async fn login(data: LoginRequest) -> Result<AuthResponse, reqwest::Error> { 
    let client = Client::new();
    let response = client.post("api/login")
        .json(&data)
        .send()
        .await?;

    let response = response.json().await?;
    Ok(response)
}
