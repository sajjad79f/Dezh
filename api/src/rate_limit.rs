//! Rate limiter ساده برای محافظت از login در برابر brute-force
//!
//! پنجره ثابت ۶۰ ثانیه‌ای per-IP: بعد از MAX_FAILURES تلاش ناموفق،
//! تا پایان پنجره همه درخواست‌های login از آن IP با 429 رد می‌شوند.
//! با موفقیت‌آمیز بودن login، رکورد IP پاک می‌شود.
//!
//! این لایه کاملاً درون حافظه (in-memory) است و بعد از ری‌استارت پروسه
//! ریست می‌شود؛ برای استقرار تک‌نود کافی است.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// طول پنجره شمارش
const WINDOW: Duration = Duration::from_secs(60);
/// حداکثر تلاش ناموفق مجاز در هر پنجره
const MAX_FAILURES: u32 = 5;
/// سقف ایمنی برای جلوگیری از رشد بی‌حد حافظه
const MAX_TRACKED: usize = 65_536;

struct Inner {
    failures: HashMap<IpAddr, Vec<Instant>>,
}

pub struct LoginRateLimiter {
    inner: Mutex<Inner>,
}

impl LoginRateLimiter {
    fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                failures: HashMap::new(),
            }),
        }
    }

    /// نمونه سراسری (بدون نیاز به تغییر AppState)
    pub fn global() -> &'static Self {
        static LIMITER: OnceLock<LoginRateLimiter> = OnceLock::new();
        LIMITER.get_or_init(LoginRateLimiter::new)
    }

    /// آیا این IP الان مسدود است؟
    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        let mut guard = self.lock();
        Self::prune(&mut guard.failures);
        guard
            .failures
            .get(&ip)
            .is_some_and(|v| v.len() >= MAX_FAILURES as usize)
    }

    /// ثبت یک تلاش ناموفق برای این IP
    pub fn record_failure(&self, ip: IpAddr) {
        let mut guard = self.lock();
        let now = Instant::now();
        let entry = guard.failures.entry(ip).or_default();
        entry.retain(|t| now.duration_since(*t) < WINDOW);
        entry.push(now);
        // سقف اضطراری: اگر جدول بیش از حد بزرگ شد، کامل پاک کن
        if guard.failures.len() > MAX_TRACKED {
            guard.failures.clear();
        }
    }

    /// پاک کردن رکورد IP (بعد از login موفق)
    pub fn clear(&self, ip: IpAddr) {
        self.lock().failures.remove(&ip);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // در صورت poison شدن Mutex، وضعیت فعلی را بازیابی کن
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn prune(map: &mut HashMap<IpAddr, Vec<Instant>>) {
        let now = Instant::now();
        map.retain(|_, attempts| {
            attempts.retain(|t| now.duration_since(*t) < WINDOW);
            !attempts.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn ip(n: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, n))
    }

    #[test]
    fn blocks_after_max_failures() {
        let lim = LoginRateLimiter::new();
        let a = ip(1);
        assert!(!lim.is_blocked(a));
        for _ in 0..MAX_FAILURES {
            lim.record_failure(a);
        }
        assert!(lim.is_blocked(a));
        // IP دیگر مسدود نیست
        assert!(!lim.is_blocked(ip(2)));
    }

    #[test]
    fn clear_unblocks() {
        let lim = LoginRateLimiter::new();
        let a = ip(3);
        for _ in 0..MAX_FAILURES {
            lim.record_failure(a);
        }
        assert!(lim.is_blocked(a));
        lim.clear(a);
        assert!(!lim.is_blocked(a));
    }

    #[test]
    fn socket_addr_ip_helper() {
        let sa = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 5), 55555);
        assert_eq!(IpAddr::V4(*sa.ip()), SocketAddr::V4(sa).ip());
    }
}