use js_sys::Math;
use web_sys::window;

pub fn string_or_none(s: String) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn generate_password(length: usize) -> String {
    const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()";

    (0..length)
        .map(|_| {
            let idx = (Math::random() * CHARSET.len() as f64) as usize;
            CHARSET.chars().nth(idx).unwrap_or('a')
        })
        .collect()
}

pub fn copy_to_clipboard(text: &str) {
    let window = window().expect("No window found.");
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();
    
    let _ = clipboard.write_text(text);
}
