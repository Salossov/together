CREATE TABLE IF NOT EXISTS users (
    id              BIGSERIAL    PRIMARY KEY,
    username        TEXT         NOT NULL UNIQUE,
    password_hash   TEXT         NOT NULL,
    email           TEXT         UNIQUE,
    internal_uuid   TEXT         NOT NULL UNIQUE,
    skin_base64     TEXT,
    skin_signature  TEXT,
    auth_type       TEXT         NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_username      ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_internal_uuid ON users(internal_uuid);
