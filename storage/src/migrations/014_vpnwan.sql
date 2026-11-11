-- N3: VPN WAN connections (PPTP / L2TP) — Dezh network module
CREATE TABLE IF NOT EXISTS vpn_connections (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL UNIQUE,
    kind          TEXT NOT NULL,            -- 'pptp' | 'l2tp'
    server        TEXT NOT NULL,            -- host or ip, optional :port
    username      TEXT NOT NULL,
    password      TEXT NOT NULL DEFAULT '',
    mtu           INT NOT NULL DEFAULT 1400,
    default_route BOOLEAN NOT NULL DEFAULT FALSE,
    keepalive     INT NOT NULL DEFAULT 10,
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    description   TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
