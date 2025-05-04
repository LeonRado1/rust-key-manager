CREATE DATABASE key_manager;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE key_types (
    id SERIAL PRIMARY KEY,
    key_type VARCHAR(15) NOT NULL,
)

CREATE TABLE keys (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_name VARCHAR(255) NOT NULL,
    key_value TEXT NOT NULL,
    key_description TEXT,
    key_type_id INTEGER NOT NULL REFERENCES key_types(id),
    key_tag VARCHAR(255),
    key_pair_value TEXT,
    expiration_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    salt TEXT,
    nonce TEXT
);

CREATE TABLE recovery_codes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR(15) NOT NULL,
    is_used BOOLEAN DEFAULT(false)
)
