use garde::Validate;

#[derive(Validate)]
pub struct ValidateEmail {
    #[garde(email)]
   pub email: String,
}

#[derive(Validate)]
pub struct ValidatePassword {
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Validate)]
pub struct ValidateUsername {
    #[garde(length(min = 2, max = 32))]
    pub username: String,
}
