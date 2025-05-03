use reqwest::Client;
use crate::models::key::PartialKey;

pub async fn get_keys(token: &str) -> Result<Vec<PartialKey>, reqwest::Error> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/keys")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let response = response.json().await?;
    Ok(response)
}
