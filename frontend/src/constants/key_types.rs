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
