use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    Allow,
    Deny,
    Drop,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleDirection {
    Inbound,
    Outbound,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Any,
    Tcp,
    Udp,
    Icmp,
}

// ─── فاز ۴: پنجره‌ی زمانی قانون ─────────────────────────

/// پنجره‌ی زمانی فعال بودن قانون — همه‌ی زمان‌ها «ساعت محلی سرور»اند
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSchedule {
    /// "HH:MM" — شروع پنجره (شامل خودش)
    pub start: String,
    /// "HH:MM" — پایان پنجره (شامل نمی‌شود؛ 23:59 یعنی تا آخر شب)
    pub end: String,
    /// نام کوتاه روزها به ترتیب هفته: sun, mon, tue, wed, thu, fri, sat
    pub days: Vec<String>,
}

impl RuleSchedule {
    pub const DAY_NAMES: [&'static str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

    /// ساخت با اعتبارسنجی کامل — خروجی همیشه کانونی است
    /// (روزها حرف کوچک، بدون تکرار، مرتب بر اساس ترتیب هفته)
    pub fn new(start: &str, end: &str, days: Vec<String>) -> Result<Self, String> {
        let start = start.trim().to_string();
        let end = end.trim().to_string();
        Self::parse_hhmm(&start)
            .ok_or_else(|| format!("invalid schedule start '{start}' (use HH:MM)"))?;
        Self::parse_hhmm(&end)
            .ok_or_else(|| format!("invalid schedule end '{end}' (use HH:MM)"))?;

        let mut picked: Vec<String> = Vec::new();
        for name in &days {
            let Some(idx) = Self::day_index(name) else {
                return Err(format!("invalid schedule day '{name}' (use sun..sat)"));
            };
            if !picked.iter().any(|d| d == Self::DAY_NAMES[idx]) {
                picked.push(Self::DAY_NAMES[idx].to_string());
            }
        }
        if picked.is_empty() {
            return Err("schedule needs at least one day".into());
        }
        picked.sort_by_key(|d| Self::day_index(d).unwrap_or(7));

        Ok(Self { start, end, days: picked })
    }

    /// "HH:MM" → دقیقه از نیمه‌شب
    fn parse_hhmm(s: &str) -> Option<u32> {
        let (h, m) = s.split_once(':')?;
        let h: u32 = h.parse().ok()?;
        let m: u32 = m.parse().ok()?;
        if h > 23 || m > 59 {
            return None;
        }
        Some(h * 60 + m)
    }

    fn day_index(name: &str) -> Option<usize> {
        Self::DAY_NAMES
            .iter()
            .position(|d| d.eq_ignore_ascii_case(name.trim()))
    }

    /// آیا در این لحظه‌ی مشخص از هفته فعال است؟
    /// `mins` = دقیقه از نیمه‌شب (0..1439)، `dow` = 0=Sun .. 6=Sat
    pub fn is_active_at(&self, mins: u32, dow: usize) -> bool {
        let day_on =
            |idx: usize| self.days.iter().any(|d| Self::day_index(d) == Some(idx));
        let Some(start) = Self::parse_hhmm(&self.start) else {
            return false;
        };
        let Some(end) = Self::parse_hhmm(&self.end) else {
            return false;
        };

        if start == end {
            // پنجره‌ی صفر = کل روزِ انتخاب‌شده
            return day_on(dow);
        }
        if start < end {
            // پنجره‌ی همان روز
            day_on(dow) && mins >= start && mins < end
        } else {
            // پنجره از نیمه‌شب می‌گذرد (مثل 22:00 تا 06:00):
            //  - بخش اول امروز از start تا نیمه‌شب
            //  - بخش دوم متعلق به پنجره‌ی دیروز است (نیمه‌شب تا end)
            (day_on(dow) && mins >= start) || (day_on((dow + 6) % 7) && mins < end)
        }
    }

    /// ساعت محلی سرور ملاک است
    pub fn is_active_now(&self) -> bool {
        use chrono::{Datelike, Local, Timelike};
        let now = Local::now();
        let mins = now.hour() * 60 + now.minute();
        let dow = now.weekday().num_days_from_sunday() as usize;
        self.is_active_at(mins, dow)
    }
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: Uuid,
    pub name: String,
    pub action: RuleAction,
    pub direction: RuleDirection,
    pub protocol: Protocol,
    pub source: String,      // IP یا CIDR یا "any"
    pub destination: String, // IP یا CIDR یا "any"
    pub port: Option<u16>,
    pub enabled: bool,
    pub priority: u32, // عدد کمتر = اولویت بالاتر
    // ── NGFW ─────────────────────────────────────────────
    pub description: String,
    /// اگر روشن باشد، هر hit این قانون در kernel log ثبت می‌شود
    /// (قابل مشاهده با `journalctl -k | grep dezh:`)
    pub log: bool,
    /// ارجاع به Address List به‌جای source/destination متنی
    pub source_set: Option<String>,
    pub dest_set: Option<String>,
    /// فاز ۳: سقف تطبیق این قانون در ثانیه (nft `limit rate N/second`)
    /// بسته‌های مازاد از این قانون رد می‌شوند و به قانون بعدی/سیاست پیش‌فرض می‌رسند
    pub rate_limit: Option<u32>,
    /// فاز ۴: پنجره‌ی زمانی — None = همیشه فعال (تا وقتی enabled باشد)
    pub schedule: Option<RuleSchedule>,
    /// فاز ۵: ارجاع به شیء دامنه (FQDN) — با «نام» شیء، مثل Address List
    /// اولویت تطبیق در موتور: fqdn > set > مقدار متنی
    pub source_fqdn: Option<String>,
    pub dest_fqdn: Option<String>,
}

impl FirewallRule {
    pub fn new(
        name: impl Into<String>,
        action: RuleAction,
        direction: RuleDirection,
        protocol: Protocol,
        source: impl Into<String>,
        destination: impl Into<String>,
        port: Option<u16>,
        priority: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            action,
            direction,
            protocol,
            source: source.into(),
            destination: destination.into(),
            port,
            enabled: true,
            priority,
            description: String::new(),
            log: false,
            source_set: None,
            dest_set: None,
            rate_limit: None,
            schedule: None,
            source_fqdn: None,
            dest_fqdn: None,
        }
    }

    /// فاز ۴: آیا این قانون همین حالا باید در nftables حضور داشته باشد؟
    /// قانونِ زمان‌دارِ خارج از پنجره → false (در apply نادیده گرفته می‌شود)
    pub fn is_effectively_active(&self) -> bool {
        self.enabled
            && self
                .schedule
                .as_ref()
                .map(|s| s.is_active_now())
                .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sched(start: &str, end: &str, days: &[&str]) -> RuleSchedule {
        RuleSchedule::new(start, end, days.iter().map(|s| s.to_string()).collect())
            .expect("valid schedule")
    }

    #[test]
    fn schedule_validation() {
        assert!(RuleSchedule::new("08:00", "17:00", vec!["mon".into()]).is_ok());
        assert!(RuleSchedule::new("25:00", "17:00", vec!["mon".into()]).is_err());
        assert!(RuleSchedule::new("08:00", "17:60", vec!["mon".into()]).is_err());
        assert!(RuleSchedule::new("08:00", "17:00", vec![]).is_err());
        assert!(RuleSchedule::new("08:00", "17:00", vec!["funday".into()]).is_err());
        // حروف بزرگ و ترتیب نامرتب قابل قبول‌اند — خروجی کانونی می‌شود
        let s = sched("08:00", "17:00", &["WED", "Mon"]);
        assert_eq!(s.days, vec!["mon", "wed"]);
    }

    #[test]
    fn weekday_window() {
        let s = sched("08:00", "18:00", &["mon", "tue"]);
        assert!(s.is_active_at(8 * 60, 1)); // دوشنبه 08:00
        assert!(s.is_active_at(9 * 60 + 30, 2)); // سه‌شنبه 09:30
        assert!(!s.is_active_at(18 * 60, 1)); // پایان پنجره شامل نمی‌شود
        assert!(!s.is_active_at(7 * 60 + 59, 1)); // قبل از شروع
        assert!(!s.is_active_at(10 * 60, 0)); // یکشنبه — روز خاموش
    }

    #[test]
    fn overnight_window() {
        // پنجشنبه 22:00 تا جمعه 06:00
        let s = sched("22:00", "06:00", &["thu"]);
        assert!(s.is_active_at(23 * 60, 4)); // پنجشنبه 23:00
        assert!(s.is_active_at(5 * 60, 5)); // جمعه 05:00 — از شب پنجشنبه
        assert!(!s.is_active_at(5 * 60, 4)); // پنجشنبه 05:00 — پنجره هنوز شروع نشده
        assert!(!s.is_active_at(7 * 60, 5)); // جمعه 07:00 — پنجره بسته شده
    }

    #[test]
    fn zero_window_means_all_day() {
        let s = sched("00:00", "00:00", &["sun"]);
        assert!(s.is_active_at(0, 0));
        assert!(s.is_active_at(23 * 60 + 59, 0));
        assert!(!s.is_active_at(12 * 60, 1));
    }

    #[test]
    fn effectively_active_respects_enabled() {
        let mut r = FirewallRule::new(
            "t",
            RuleAction::Allow,
            RuleDirection::Inbound,
            Protocol::Any,
            "any",
            "any",
            None,
            None,
            None,
            100,
        );
        assert!(r.is_effectively_active());
        r.enabled = false;
        assert!(!r.is_effectively_active());
        r.enabled = true;
        r.schedule = Some(sched("08:00", "09:00", &["mon"]));
        // نتیجه به ساعت فعلی سرور بستگی دارد — فقط نباید panic کند
        let _ = r.is_effectively_active();
    }
}