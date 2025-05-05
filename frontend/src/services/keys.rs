use js_sys::Uint8Array;
use reqwest::Client;
use reqwest::multipart::Form;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FormData};
use crate::helpers::error_codes::get_error_message_from_code;
use crate::models::key::{KeyRequest, PartialKey};

pub async fn get_keys(token: &str) -> Result<Vec<PartialKey>, String> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/keys")
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

pub async fn add_key(token: &str, key_request: KeyRequest) -> Result<(), String> {
    let client = Client::new();
    
    let response = client.post("http://127.0.0.1:8000/keys")
        .header("Authorization", format!("Bearer {}", token))
        .json(&key_request)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;


    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    Ok(())
}

pub async fn import_ssh_key(
    token: &str,
    key_request: KeyRequest,
    file: File
) -> Result<(), String> {
    let client = Client::new();

    // Get the file data
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| "Failed to read the file.")?;

    let uint8_array = Uint8Array::new(&array_buffer);
    let file_data = uint8_array.to_vec();
    
    let form = Form::new()
        .text("json", serde_json::to_string(&key_request)
            .map_err(|_| "Error while serializing JSON.")?)
        .part("file", reqwest::multipart::Part::bytes(file_data)
            .file_name(file.name()));

    let response = client.post("http://127.0.0.1:8000/keys/import")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|_e| "Error while sending request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    Ok(())
}
