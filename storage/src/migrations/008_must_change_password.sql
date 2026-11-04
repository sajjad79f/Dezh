-- فاز ۲ امنیت: اجبار تغییر رمز عبور
-- پرچم must_change_password برای کاربران جدید (به‌ویژه admin بوت‌استرپ)
-- تا در اولین ورود، UI تغییر رمز را اجباری کند.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS must_change_password BOOLEAN NOT NULL DEFAULT FALSE;