-- Migration cho module auth-login: quản lý khóa tài khoản khi thử sai
CREATE TABLE IF NOT EXISTS m_auth_login_attempt (
    key TEXT PRIMARY KEY NOT NULL,
    failed_count INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    updated_at TEXT NOT NULL
);
