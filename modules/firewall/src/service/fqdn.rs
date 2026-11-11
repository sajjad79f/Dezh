//! فاز ۵ — سرویس Resolver اشیای دامنه (FQDN)
//!
//! ترد اختصاصی `dezh-fqdn` که هر بازه (کلید firewall_settings:
//! fqdn_resolve_interval — پیش‌فرض ۳۰۰ ثانیه) تمام اشیای فعال را
//! resolve می‌کند و کش IP را در جدول fqdn_objects به‌روز نگه می‌دارد.
//!
//! سیاست شکست: اگر DNS موقتاً خطا بدهد، IPهای قبلی حفظ می‌شوند
//! (fail-open کش) و فقط last_error ثبت می‌شود. اگر کشِ شیئی تغییر
//! کند، apply_rules دوباره صدا زده می‌شود تا ruleset تازه شود.

use std::net::ToSocketAddrs;
use std::time::Duration;

use shared_kernel::runtime::block_on;
use storage::{DbPool, FqdnObjectRepo, SettingsRepo};

use super::FirewallService;

const DEFAULT_INTERVAL_SECS: u64 = 300;
const MIN_INTERVAL_SECS: u64 = 30;
const MAX_INTERVAL_SECS: u64 = 86_400;

/// شروع ترد resolver — از attach_pool صدا زده می‌شود
pub fn spawn(service: FirewallService, pool: DbPool) {
    let _ = std::thread::Builder::new()
        .name("dezh-fqdn".into())
        .spawn(move || run_loop(service, pool));
}

fn run_loop(service: FirewallService, pool: DbPool) {
    eprintln!("[dezh-fqdn] resolver thread started");
    loop {
        // بازه در هر چرخه دوباره خوانده می‌شود تا تغییر تنظیمات
        // بدون ری‌استارت اعمال شود
        let interval = read_interval(&pool);

        let changed = resolve_all(&pool);
        if changed {
            if let Err(e) = service.apply_rules() {
                eprintln!("[dezh-fqdn] apply_rules after resolve failed: {e}");
            }
        }

        std::thread::sleep(Duration::from_secs(interval));
    }
}

/// بازه resolve از تنظیمات — پیش‌فرض ۳۰۰ ثانیه، محدود بین ۳۰ ثانیه و ۲۴ ساعت
fn read_interval(pool: &DbPool) -> u64 {
    let repo = SettingsRepo::new(pool.inner());
    block_on(repo.get("fqdn_resolve_interval"))
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS)
        .clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS)
}

/// resolve همه‌ی اشیای فعال — خروجی: آیا کشِ حداقل یک شیء تغییر کرد؟
pub(crate) fn resolve_all(pool: &DbPool) -> bool {
    let repo = FqdnObjectRepo::new(pool.inner());
    let objects = match block_on(repo.list_enabled()) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[dezh-fqdn] list enabled objects failed: {e}");
            return false;
        }
    };
    if objects.is_empty() {
        return false;
    }

    let mut changed = false;
    for obj in objects {
        // از wildcard، فقط دامنه‌ی پایه resolve می‌شود
        let base = obj.base_domain();
        match resolve_host(base) {
            Ok(mut ips) => {
                ips.sort();
                ips.dedup();

                if ips != obj.resolved {
                    changed = true;
                }
                if let Err(e) = block_on(repo.set_resolved(&obj.name, &ips, None)) {
                    eprintln!("[dezh-fqdn] persist '{}' failed: {e}", obj.name);
                }
            }
            Err(e) => {
                // خطای موقت DNS → IPهای قبلی حفظ می‌شوند (fail-open کش)
                eprintln!("[dezh-fqdn] resolve '{base}' failed: {e}");
                if let Err(pe) = block_on(repo.set_resolved(&obj.name, &obj.resolved, Some(&e))) {
                    eprintln!("[dezh-fqdn] persist error for '{}' failed: {pe}", obj.name);
                }
            }
        }
    }
    changed
}

/// resolve یک هاست — همه‌ی IPهای برگشتی (v4/v6) به‌صورت رشته
fn resolve_host(host: &str) -> Result<Vec<String>, String> {
    let addrs: Vec<String> = (host, 0u16)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .map(|a| a.ip().to_string())
        .collect();

    if addrs.is_empty() {
        return Err("resolver returned no addresses".into());
    }
    Ok(addrs)
}