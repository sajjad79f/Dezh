CREATE TABLE IF NOT EXISTS nat_rules (
    id           UUID PRIMARY KEY,
    name         TEXT NOT NULL,
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    kind         TEXT NOT NULL,          -- masquerade | dnat
    interface    TEXT NOT NULL,          -- oif (masq) یا iif (dnat)
    source       TEXT NOT NULL DEFAULT 'any',
    destination  TEXT NOT NULL DEFAULT 'any',
    protocol     TEXT NOT NULL DEFAULT 'any',  -- any | tcp | udp
    dest_port    INTEGER,                -- پورت خارجی (dnat)
    target       TEXT,                   -- IP یا IP:port داخلی (dnat)
    description  TEXT NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nat_rules_kind ON nat_rules (kind);