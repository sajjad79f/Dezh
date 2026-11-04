-- فاز ۴: زمان‌بندی زمانی قوانین فایروال (Time-based Rules)
-- پنجره‌ی فعال بودن قانون بر اساس ساعت محلی سرور است
ALTER TABLE firewall_rules
    ADD COLUMN IF NOT EXISTS schedule_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS schedule_start   TEXT    NOT NULL DEFAULT '00:00',
    ADD COLUMN IF NOT EXISTS schedule_end     TEXT    NOT NULL DEFAULT '23:59',
    ADD COLUMN IF NOT EXISTS schedule_days    TEXT    NOT NULL DEFAULT 'sun,mon,tue,wed,thu,fri,sat';