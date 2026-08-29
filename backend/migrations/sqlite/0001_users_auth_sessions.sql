-- 0001: users and authentication sessions.
-- Roles/statuses are TEXT + CHECK so the Rust enums map by string.

CREATE TABLE users (
    id                TEXT PRIMARY KEY,
    firebase_uid      TEXT NOT NULL UNIQUE,
    email             TEXT,
    display_name      TEXT,
    photo_url         TEXT,
    phone             TEXT,
    role              TEXT NOT NULL DEFAULT 'customer'
        CHECK (role IN ('customer', 'mechanic', 'admin')),
    status            TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'suspended', 'deleted')),
    onboarding_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (onboarding_status IN ('pending', 'complete')),
    created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Application auth sessions. The raw refresh token NEVER lands here, only
-- its SHA-256 hash. Rotation replaces the hash; revocation sets revoked_at.
CREATE TABLE auth_sessions (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    created_at         TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at         TEXT NOT NULL,
    last_used_at       TEXT,
    revoked_at         TEXT,
    user_agent         TEXT,
    ip_address         TEXT
);

CREATE INDEX auth_sessions_user_id_idx ON auth_sessions (user_id);
CREATE INDEX auth_sessions_expires_at_idx ON auth_sessions (expires_at);
