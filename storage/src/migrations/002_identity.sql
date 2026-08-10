-- ── کاربران مدیریتی پنل (DCM) ───────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name  TEXT,
    role          TEXT NOT NULL DEFAULT 'viewer',  -- admin | operator | viewer
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);

-- نشست‌های پنل (اختیاری برای فاز بعد login)
CREATE TABLE IF NOT EXISTS user_sessions (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    ip_address    TEXT,
    user_agent    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMPTZ NOT NULL,
    revoked_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions (token_hash);

-- ── هویت شبکه (AAA / Accounting) ───────────────────────
CREATE TABLE IF NOT EXISTS identities (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    display_name  TEXT,
    source        TEXT NOT NULL DEFAULT 'local',  -- local | ldap | ad
    external_id   TEXT,                          -- DN یا objectGUID در AD
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    metadata      JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_identities_username ON identities (username);
CREATE INDEX IF NOT EXISTS idx_identities_source ON identities (source);

CREATE TABLE IF NOT EXISTS identity_groups (
    id            UUID PRIMARY KEY,
    name          TEXT NOT NULL UNIQUE,
    description   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS identity_group_members (
    group_id      UUID NOT NULL REFERENCES identity_groups(id) ON DELETE CASCADE,
    identity_id   UUID NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, identity_id)
);

-- نشست‌های اکانتینگ شبکه
CREATE TABLE IF NOT EXISTS accounting_sessions (
    id               UUID PRIMARY KEY,
    identity_id      UUID REFERENCES identities(id) ON DELETE SET NULL,
    protocol         TEXT NOT NULL,          -- vpn | nac | portal | 8021x | other
    session_id       TEXT,                   -- شناسه سمت NAS/VPN
    ip_address       TEXT,
    mac_address      TEXT,
    nas_ip           TEXT,
    started_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at         TIMESTAMPTZ,
    bytes_in         BIGINT NOT NULL DEFAULT 0,
    bytes_out        BIGINT NOT NULL DEFAULT 0,
    terminate_cause  TEXT,
    metadata         JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_accounting_identity ON accounting_sessions (identity_id);
CREATE INDEX IF NOT EXISTS idx_accounting_started ON accounting_sessions (started_at DESC);
CREATE INDEX IF NOT EXISTS idx_accounting_active ON accounting_sessions (ended_at) WHERE ended_at IS NULL;