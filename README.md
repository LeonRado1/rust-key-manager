# 🔐 Key Manager
A secure and efficient key management system built with Rust for managing passwords, API keys, tokens, and SSH keys.

### Overview
Key Manager is a full-stack web application that provides secure storage and management of sensitive credentials. It allows users to safely generate, store, encrypt, and manage various types of keys and passwords with a user-friendly interface.

## Main Features

### 🔑 Key Management
- Store and manage multiple credential types:
  - Passwords (with validation requirements)
  - API Keys
  - Access Tokens
  - SSH Keys (with import/export functionality)
- Automatic key generation support
- Key rotation and expiration management
- Key revocation system
- Custom tags and descriptions

### 🛡️ Security Features
- User's master password for key encryption
- AES-GCM encryption for all stored credentials
- JWT token-based authentication
- Rate limiting middleware for brute force protection
- Parameterized SQL queries to prevent SQL injection
- Input validation and sanitization

### 📧 Email Service Integration
- Welcome emails with recovery codes
- Security alerts for:
  - Password changes
  - Recovery code usage
- Expiration notifications for keys
 
### ⏰ Automated Tasks
- Scheduler for checking key expiration
- Periodic email notifications for expiring keys

## Security Flow
1. User registration creates recovery codes
2. Master password hashed with bcrypt
3. Keys encrypted with AES-GCM using master password
4. JWT tokens for session management
5. Rate limiting for login and change password attempts
6. Parameterized queries for database operations
7. Email notificatons for suspicious activity
 
## Tech Stack

### 🔧 Technologies Used
> Backend
- Rust
- Rocket (Web framework)
- PostgreSQL with SQLx
> Frontend
- Rust
- Yew (WebAssembly framework)
- Bootstrap CSS

### 📦 Key Dependencies
> Backend
```
rocket = { version = "0.5.1", features = ["json"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
aes-gcm = "0.10.2"
jsonwebtoken = "9.3.1"
bcrypt = "0.17.0"
ssh-key = { version = "0.6.1", features = ["ed25519"] }
```
> Frontend
```
yew = { features = ["csr"] }
wasm-bindgen = "0.2"
reqwest = { version = "0.11", features = ["json"] }
```

### 🚀 Running the Application
1. Clone the repository
```
https://github.com/LeonRado1/rust-key-manager.git
cd rust-key-manager
```
> Running Backend
```
cd backend
```
2. Create .env file in the project root
```
DATABASE_URL=postgres://username:password@localhost/keymanager
JWT_SECRET=your_jwt_secret
SMTP_SERVER=your_smtp_server
SMTP_PORT=587
```
3. Run database migrations
> [!NOTE]
> Migration file can be found inside ***/database*** folder
4. Start the Rocket server
```
cargo run
```
> Running Frontend
```
cd frontend
```
5. Install trunk (WebAssembly bundler)
```
cargo install trunk
```
6. Add WebAssembly target
```
rustup target add wasm32-unknown-unknown
```
7. Start the development server
```
trunk serve
```

## 🦀 Showcase
> Dashboard

![](https://github.com/user-attachments/assets/57c0e122-2a59-4c95-b02a-932d11c8f4d0)

> Add Key

![](https://github.com/user-attachments/assets/68e25bcb-f259-44ec-9ee5-a00cdb8d112f)

> Key Detail

![](https://github.com/user-attachments/assets/8debe1ed-8008-452a-a5e1-abd79c85cd14)

## Authors
Leon Rado, Dokaniev Andrii
