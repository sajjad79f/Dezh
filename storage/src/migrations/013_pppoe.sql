-- N2: PPPoE connection profiles (Dezh network module)
CREATE TABLE IF NOT EXISTS pppoe_profiles (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL UNIQUE,
    parent_iface  TEXT NOT NULL,
    username      TEXT NOT NULL,
    password      TEXT NOT NULL DEFAULT '',
    mtu           INT NOT NULL DEFAULT 1492,
    default_route BOOLEAN NOT NULL DEFAULT TRUE,
    keepalive     INT NOT NULL DEFAULT 10,
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    description   TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
