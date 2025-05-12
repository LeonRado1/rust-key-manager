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
    key_type VARCHAR(15) NOT NULL
);

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
    updated_at TIMESTAMP,
    salt TEXT NOT NULL,
    nonce TEXT NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE recovery_codes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR(15) NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT false
);

-- Indexes for the users table
CREATE INDEX idx_users_email ON users(email);

-- Indexes for the recovery_codes table
CREATE INDEX idx_recovery_codes_user_unused ON recovery_codes(user_id, is_used)
    WHERE is_used = false;

-- Indexes for the keys table
CREATE INDEX idx_keys_user_id ON keys(user_id);

-- Indexes for the active keys
CREATE INDEX idx_keys_user_active ON keys(user_id)
    WHERE is_revoked = false;

-- Indexes for the sorting
CREATE INDEX idx_keys_updated_at ON keys(updated_at DESC NULLS LAST);

-- Init data for key_types
INSERT INTO key_types (id, key_type) VALUES
(1, 'PASSWORD'),
(2, 'TOKEN'),
(3, 'API_KEY'),
(4, 'SSH_KEY');

