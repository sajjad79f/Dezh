-- Phase 3: Security Events + per-rule rate limit
-- ۱) ستون rate_limit روی قوانین فایروال (سقف تطبیق در ثانیه — NULL = بی‌سقف)
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS rate_limit INTEGER;

-- ۲) جدول رخدادهای امنیتی (hit قوانین از kernel log + رخدادهای ورود)
CREATE TABLE IF NOT EXISTS security_events (
    id         BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    kind       TEXT NOT NULL,                    -- rule_hit | login_failed | ...
    severity   TEXT NOT NULL DEFAULT 'warning',  -- info | warning | critical
    source_ip  TEXT,
    dest_port  INTEGER,
    rule_id    UUID,
    hits       BIGINT NOT NULL DEFAULT 1,
    message    TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_sec_events_created ON security_events (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sec_events_kind    ON security_events (kind);
CREATE INDEX IF NOT EXISTS idx_sec_events_source  ON security_events (source_ip);