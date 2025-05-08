use js_sys::Uint8Array;
use reqwest::Client;
use reqwest::multipart::Form;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, File, HtmlAnchorElement, Url};
use yew::BaseComponent;
use crate::helpers::error_codes::get_error_message_from_code;
use crate::models::key::{Key, KeyRequest, PartialKey, UpdateKeyRequest};

pub async fn get_keys(token: &str) -> Result<Vec<PartialKey>, String> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/keys")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }
    
    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;
    
    Ok(response)
}

pub async fn get_revoked_keys(token: &str) -> Result<Vec<PartialKey>, String> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/keys/revoked")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn get_expired_keys(token: &str) -> Result<Vec<PartialKey>, String> {
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8000/keys/expired")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

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
        .map_err(|_e| "Error while sending a request. Try again later.")?;

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
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    Ok(())
}

pub async fn get_key_detail(token: &str, key_id: i32) -> Result<Key, String> {
    let client = Client::new();
    let response = client.get(format!("http://127.0.0.1:8000/keys/key/{}", key_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn export_key(key_value: &str) -> Result<(), String> {
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(key_value));

    let blob_options = BlobPropertyBag::new();

    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_options)
        .map_err(|_| "Failed to create a blob from the content.")?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Failed to generate a URL for the download.")?;

    let window = web_sys::window().ok_or("Failed to access the browser window.")?;
    let document = window.document().ok_or("Failed to access the document object.")?;
    let body = document.body().ok_or("Failed to access the document body.")?;

    let link = document
        .create_element("a")
        .map_err(|_| "Failed to create a download link.")?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Failed to cast the link element.")?;

    link.set_href(&url);
    link.set_download("ssh_exported.pem");

    body.append_child(&link).map_err(|_| "Failed to append link to the body.")?;
    link.click();
    body.remove_child(&link).map_err(|_| "Failed to remove the link element.")?;

    Url::revoke_object_url(&url).map_err(|_| "Failed to revoke the object URL.")?;

    Ok(())
}

pub async fn rotate_key(token: &str, key_id: i32) -> Result<Key, String> {
    let client = Client::new();

    let response = client.patch(format!("http://127.0.0.1:8000/keys/rotate/{}", key_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn revoke_key(token: &str, key_id: i32) -> Result<(), String> {
    let client = Client::new();

    let response = client.patch(format!("http://127.0.0.1:8000/keys/{}", key_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    Ok(())
}

pub async fn change_key(token: &str, key_request: UpdateKeyRequest) -> Result<Key, String> {
    let client = Client::new();

    let response = client.put("http://127.0.0.1:8000/keys/change")
        .header("Authorization", format!("Bearer {}", token))
        .json(&key_request)
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn extend_key(token: &str, key_request: UpdateKeyRequest) -> Result<Key, String> {
    let client = Client::new();

    let response = client.put("http://127.0.0.1:8000/keys/extend")
        .header("Authorization", format!("Bearer {}", token))
        .json(&key_request)
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn extend_rotate_key(token: &str, key_request: UpdateKeyRequest) -> Result<Key, String> {
    let client = Client::new();

    let response = client.put("http://127.0.0.1:8000/keys/extend/rotate")
        .header("Authorization", format!("Bearer {}", token))
        .json(&key_request)
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    let response = response.json().await
        .map_err(|_e| "Error while parsing response. Try again later.")?;

    Ok(response)
}

pub async fn delete_key(token: &str, key_id: i32) -> Result<(), String> {
    let client = Client::new();

    let response = client.delete(format!("http://127.0.0.1:8000/keys/{}", key_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_e| "Error while sending a request. Try again later.")?;

    if let Some(error_message) = get_error_message_from_code(response.status()) {
        return Err(error_message);
    }

    Ok(())
}
