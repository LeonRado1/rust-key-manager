use garde::{Report, Validate};
use ssh_key::{PrivateKey, PublicKey};

#[derive(Debug, Validate)]
pub struct UsernameRule(
    #[garde(length(min = 3, max = 30), pattern(r"^[a-zA-Z0-9_]+$"))]
    String
);

#[derive(Debug, Validate)]
pub struct EmailRule(
    #[garde(email)]
    String
);

pub fn validate_username(username: &str) -> Result<(), Report> {
    UsernameRule(username.to_string()).validate()
}

pub fn validate_email(email: &str) -> Result<(), Report> {
    EmailRule(email.to_string()).validate()
}

pub fn validate_password_requirements(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long");
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one number");
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter");
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain at least one lowercase letter");
    }

    let special_chars = "!@#$%^&*()";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err("Password must contain at least one special character");
    }

    Ok(())
}

pub fn validate_openssh_private_key(key: &str) -> bool {
    match PrivateKey::from_openssh(key) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn validate_openssh_key(key: &str) -> bool {
    match PublicKey::from_openssh(key) {
        Ok(_) => true,
        Err(_) => false
    }
}
