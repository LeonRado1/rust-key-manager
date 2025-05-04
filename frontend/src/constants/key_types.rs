pub const PASSWORD: i32 = 1;
pub const TOKEN: i32 = 2;
pub const API_KEY: i32 = 3;
pub const SSH_KEY: i32 = 4;

pub fn get_type_class(key_type: i32) -> &'static str {
    match key_type {
        PASSWORD => "bg-success",
        TOKEN => "bg-danger",
        API_KEY => "bg-warning",
        SSH_KEY => "bg-info",
        _ => "bg-secondary",
    }
}

pub fn get_btn_type_class(key_type: i32) -> String {
    format!("btn-{}", &get_type_class(key_type)[3..])
}

pub fn get_type_name(key_type: i32) -> &'static str {
    match key_type {
        PASSWORD => "Password",
        TOKEN => "Token",
        API_KEY => "API Key",
        SSH_KEY => "SSH Key",
        _ => "Unknown",
    }
}
