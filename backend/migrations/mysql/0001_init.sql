CREATE TABLE IF NOT EXISTS users (
    id              BIGINT        NOT NULL AUTO_INCREMENT PRIMARY KEY,
    username        VARCHAR(64)   NOT NULL UNIQUE,
    password_hash   VARCHAR(255)  NOT NULL,
    email           VARCHAR(255)  UNIQUE,
    internal_uuid   CHAR(36)      NOT NULL UNIQUE,
    skin_base64     LONGTEXT,
    skin_signature  TEXT,
    auth_type       VARCHAR(16)   NOT NULL,
    created_at      DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_users_username      (username),
    INDEX idx_users_internal_uuid (internal_uuid)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
