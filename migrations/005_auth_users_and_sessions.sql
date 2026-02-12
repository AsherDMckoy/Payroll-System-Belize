-- Application authentication users + sessions

CREATE TABLE IF NOT EXISTS app_users (
    id              SERIAL PRIMARY KEY,
    username        VARCHAR(100) NOT NULL UNIQUE,
    full_name       VARCHAR(160) NOT NULL,
    role            VARCHAR(20) NOT NULL CHECK (role IN ('admin', 'payroll', 'viewer')),
    password_hash   TEXT NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id                  SERIAL PRIMARY KEY,
    user_id             INTEGER NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    session_token_hash  CHAR(64) NOT NULL UNIQUE,
    expires_at          TIMESTAMP NOT NULL,
    revoked_at          TIMESTAMP,
    ip_address          VARCHAR(64),
    user_agent          TEXT,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_app_users_active ON app_users(is_active);
