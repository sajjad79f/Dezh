//! فاز ۴ — زمان‌بند قوانین فایروال (Time-based Rules)
//!
//! قوانینی که `schedule` دارند فقط داخل پنجره‌ی زمانی خودشان باید در
//! ruleset باشند. به‌جای rebuild دائمی، این ماژول یک thread سبک می‌سازد:
//!
//! ۱. هر ۲۰ ثانیه «امضای» وضعیت قوانین زمان‌دار حساب می‌شود
//!    (برای هر قانون زمان‌دار: داخل پنجره است یا نه)
//! ۲. اگر امضا نسبت به چک قبلی عوض شود یعنی حداقل یک قانون وارد یا
//!    خارج پنجره شده → `apply_rules` دوباره زده می‌شود
//!
//! نکته‌ها:
//! - ساعت ملاک، ساعت سرور است (نه مرورگر کلاینت).
//! - پنجره‌های شب‌گذر (مثل 22:00 تا 06:00) خودشان در `RuleSchedule`
//!   هندل می‌شوند؛ اینجا فقط وضعیت فعلی پرسیده می‌شود.
//! - تغییر ساعت سیستم (NTP / تغییر دستی) هم به‌طور طبیعی پوشش داده
//!   می‌شود، چون امضا از زمان واقعی سرور ساخته می‌شود.

use std::time::Duration;

use crate::models::FirewallRule;

use super::FirewallService;

/// دوره‌ی چک وضعیت پنجره‌های زمانی
const CHECK_INTERVAL: Duration = Duration::from_secs(20);

/// امضای وضعیت قوانین زمان‌دار — برای هر قانون زمان‌دار:
/// `<uuid>=<داخل پنجره؟>`؛ تغییر امضا یعنی عبور از مرز پنجره
fn time_signature(fw: &FirewallService) -> String {
    let rules = fw.list_rules();
    let mut scheduled: Vec<&FirewallRule> =
        rules.iter().filter(|r| r.schedule.is_some()).collect();
    scheduled.sort_by(|a, b| a.id.cmp(&b.id));

    scheduled
        .iter()
        .map(|r| {
            let open = r.schedule.as_ref().map_or(false, |s| s.is_active_now());
            format!("{}={}", r.id, open)
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// spawn کردن thread زمان‌بند — از `attach_pool` صدا زده می‌شود
pub fn spawn_scheduler(fw: FirewallService) {
    let spawned = std::thread::Builder::new()
        .name("dezh-sched".into())
        .spawn(move || run(fw));

    if spawned.is_err() {
        eprintln!("[firewall] failed to spawn schedule thread");
    }
}

fn run(fw: FirewallService) {
    eprintln!(
        "[firewall] schedule thread started (check every {}s)",
        CHECK_INTERVAL.as_secs()
    );

    let mut prev = time_signature(&fw);
    loop {
        std::thread::sleep(CHECK_INTERVAL);

        let cur = time_signature(&fw);
        if cur == prev {
            continue;
        }

        prev = cur;
        eprintln!("[firewall] schedule boundary crossed — re-applying rules");

        if let Err(e) = fw.apply_rules() {
            eprintln!("[firewall] scheduled apply_rules failed: {e}");
        }
    }
}