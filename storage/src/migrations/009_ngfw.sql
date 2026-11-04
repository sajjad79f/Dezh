-- 009: NGFW — ستون‌های جدید قانون + Address Lists + تنظیمات فایروال

-- قانون فایروال: توضیحات، لاگ‌گیری، ارجاع به Address List
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '';
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS log BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS source_set TEXT;
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS dest_set TEXT;

-- Address Lists (معادل nftables set)
-- kind = blacklist | whitelist
CREATE TABLE IF NOT EXISTS ip_sets (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    kind        TEXT NOT NULL CHECK (kind IN ('blacklist', 'whitelist')),
    description TEXT NOT NULL DEFAULT '',
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    entries     TEXT[] NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- تنظیمات فایروال (کلید/مقدار) — مثل dos_protection و syn_rate
CREATE TABLE IF NOT EXISTS firewall_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);