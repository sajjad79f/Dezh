CREATE TABLE IF NOT EXISTS static_routes (
    id           UUID PRIMARY KEY,
    destination  TEXT NOT NULL,
    gateway      TEXT,
    device       TEXT,
    description  TEXT NOT NULL DEFAULT '',
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_static_routes_dest
    ON static_routes (destination, COALESCE(gateway, ''), COALESCE(device, ''));