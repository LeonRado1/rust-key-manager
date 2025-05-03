use web_sys::window;

pub fn save_token(token: &str) -> Result<(), String> {
    let window = window().ok_or("No window found.")?;
    
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage API error.")?
        .ok_or("localStorage not available.")?;

    storage.set_item("token", token)
        .map_err(|_| "Failed to save token.")?;

    Ok(())
}

pub fn get_token() -> Result<String, String> {
    let window = window().ok_or("No window found.")?;
    
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage API error.")?
        .ok_or("localStorage not available.")?;

    storage.get_item("token")
        .map_err(|_| "Failed to get token.")?
        .ok_or("Token not found.".to_string())
}

pub fn remove_token() -> Result<(), String> {
    let window = window().ok_or("No window found.")?;
    
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage API error.")?
        .ok_or("localStorage not available.")?;

    storage
        .remove_item("token")
        .map_err(|_| "Failed to remove token.".to_string())?;

    Ok(())
}
