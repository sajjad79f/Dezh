-- Assets (DAI)
CREATE TABLE IF NOT EXISTS assets (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    asset_type  TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'unknown',
    metadata    JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_assets_name ON assets (name);

-- Firewall rules
CREATE TABLE IF NOT EXISTS firewall_rules (
    id           UUID PRIMARY KEY,
    name         TEXT NOT NULL,
    action       TEXT NOT NULL,
    direction    TEXT NOT NULL,
    protocol     TEXT NOT NULL,
    source       TEXT NOT NULL DEFAULT 'any',
    destination  TEXT NOT NULL DEFAULT 'any',
    port         INTEGER,
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    priority     INTEGER NOT NULL DEFAULT 100,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_firewall_rules_priority ON firewall_rules (priority);

-- Audit log (DCM / مدیریتی)
CREATE TABLE IF NOT EXISTS audit_log (
    id          UUID PRIMARY KEY,
    actor       TEXT,
    action      TEXT NOT NULL,
    resource    TEXT,
    detail      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log (created_at DESC);