ALTER TABLE accounting_sessions
    ADD COLUMN IF NOT EXISTS last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE INDEX IF NOT EXISTS idx_accounting_last_updated
    ON accounting_sessions (last_updated)
    WHERE ended_at IS NULL;