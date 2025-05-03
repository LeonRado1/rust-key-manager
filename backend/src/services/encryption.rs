use aes_gcm::*;
use aes_gcm::aead::Aead;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::rand::SecureRandom;
use rocket::http::Status;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use ssh_key::HashAlg::Sha256;

pub struct EncryptedData {
    pub ciphertext: String,
    pub salt: String,
    pub nonce: String,
}

pub struct SshKeyPair {
    pub private_key: String,
    pub public_key: String,
}


pub fn encrypt(value: &str, master_password: &str) -> Result<EncryptedData, Status> {

    let mut salt = [0u8; 16];
    ring::rand::SystemRandom::new()
        .fill(&mut salt)
        .map_err(|_| Status::InternalServerError)?;

    let mut nonce_bytes = [0u8; 12];
    ring::rand::SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| Status::InternalServerError)?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut key = [0u8; 32];
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(10_000).unwrap(),
        &salt,
        master_password.as_bytes(),
        &mut key,
    );

    let cipher = Aes256Gcm::new(key.as_slice().into());
    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(EncryptedData {
        ciphertext: BASE64.encode(ciphertext),
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(nonce_bytes),
    })
}

pub fn decrypt(ciphertext: &str, salt: &str, nonce: &str, master_password: &str) -> Result<String, Status> {

    let ciphertext = BASE64.decode(ciphertext)
        .map_err(|_| Status::InternalServerError)?;

    let salt = BASE64.decode(salt)
        .map_err(|_| Status::InternalServerError)?;

    let nonce_bytes = BASE64.decode(nonce)
        .map_err(|_| Status::InternalServerError)?;

    let mut key = [0u8; 32];
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(10_000).unwrap(),
        &salt,
        master_password.as_bytes(),
        &mut key,
    );

    let cipher = Aes256Gcm::new(key.as_slice().into());
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|_| Status::InternalServerError)?;
    
    String::from_utf8(plaintext)
        .map_err(|_| Status::InternalServerError)
}

pub fn generate_ssh_key_pair() -> Result<SshKeyPair, Status> {

    let private_key = PrivateKey::random(
        &mut rand::thread_rng(), 
        Algorithm::Rsa { hash: Some(Sha256) }
    ).map_err(|_| Status::InternalServerError)?;
    
    let private_key_openssh = private_key
        .to_openssh(LineEnding::LF)
        .map_err(|_| Status::InternalServerError)?;

    let public_key_openssh = private_key
        .public_key()
        .to_openssh()
        .map_err(|_| Status::InternalServerError)?;

    Ok(SshKeyPair {
        private_key: private_key_openssh.to_string(),
        public_key: public_key_openssh.to_string(),
    })
}
