-- فاز ۵ — بخش ۳: اشیاء FQDN
-- نام دامنه به‌جای IP ثابت؛ رزولوشن دوره‌ای توسط ترد dezh-fqdn
CREATE TABLE IF NOT EXISTS fqdn_objects (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name             TEXT UNIQUE NOT NULL,
    fqdn             TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    enabled          BOOLEAN NOT NULL DEFAULT TRUE,
    resolved         TEXT[]  NOT NULL DEFAULT '{}',
    last_resolved_at TIMESTAMPTZ,
    last_error       TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ارجاع قوانین به اشیاء FQDN (با نام — مثل source_set/dest_set فاز ۲)
ALTER TABLE firewall_rules ADD COLUMN IF NOT EXISTS source_fqdn TEXT;
ALTER TABLE firewall_rules ADD COLUMN IF NOT EXISTS dest_fqdn  TEXT;