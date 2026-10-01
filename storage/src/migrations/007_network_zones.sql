CREATE TABLE IF NOT EXISTS zones (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    accounting    BOOLEAN NOT NULL DEFAULT FALSE,
    description   TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO zones (id, name, display_name, accounting, description) VALUES
  (gen_random_uuid(), 'lan',  'LAN',  FALSE, 'Internal network'),
  (gen_random_uuid(), 'wan',  'WAN',  TRUE,  'Internet uplink — accounting on'),
  (gen_random_uuid(), 'dmz',  'DMZ',  FALSE, 'Demilitarized zone')
ON CONFLICT (name) DO NOTHING;

-- سازگاری با نام جدول قدیمی
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'interfaces'
  ) AND NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'network_interfaces'
  ) THEN
    ALTER TABLE interfaces RENAME TO network_interfaces;
  END IF;
END $$;

CREATE TABLE IF NOT EXISTS network_interfaces (
    name          TEXT PRIMARY KEY,
    zone          TEXT NOT NULL DEFAULT 'lan',
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    ipv4_mode     TEXT NOT NULL DEFAULT 'none',
    address_cidr  TEXT,
    gateway       TEXT,
    description   TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS ipv4_mode TEXT NOT NULL DEFAULT 'none';
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS address_cidr TEXT;
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS gateway TEXT;
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '';
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
ALTER TABLE network_interfaces ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();