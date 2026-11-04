//! فاز ۳ — جمع‌کننده‌ی رخدادهای امنیتی از لاگ هسته (`/dev/kmsg`)
//!
//! هر قانونی که `log = true` داشته باشد هنگام apply با prefix
//! `dezh:<rule-uuid>: ` در kernel log می‌نویسد. این ماژول:
//!
//! ۱. `/dev/kmsg` را می‌خواند (نیاز به root — Dezh همین‌طور اجرا می‌شود)
//! ۲. فقط خطوط `dezh:<uuid>: ` را پارس می‌کند (SRC/DST/PROTO/DPT)
//! ۳. در پنجره‌ی ۵ ثانیه‌ای بر اساس (rule, src, proto, dport) تجمیع می‌کند
//! ۴. هر پنجره یک ردیف به جدول `security_events` می‌نویسد
//! ۵. هر ساعت رخدادهای قدیمی‌تر از ۷ روز را پاک می‌کند
//!
//! اگر `/dev/kmsg` در دسترس نباشد، کالکتور بی‌صدا غیرفعال می‌شود و
//! بقیه‌ی سیستم (از جمله صفحه‌ی Events برای رخدادهای ورود) کار می‌کند.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::time::{Duration, Instant};

use shared_kernel::runtime::block_on;
use storage::{DbPool, EventsRepo, NewSecurityEvent};
use uuid::Uuid;

/// فاصله‌ی flush به DB
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);
/// حداکثر کلیدهای تجمیع همزمان (محافظ ضد flood حافظه)
const MAX_KEYS: usize = 2000;
/// حداکثر ردیف insert در هر flush
const MAX_ROWS_PER_FLUSH: usize = 500;
/// دوره‌ی پاکسازی رخدادهای قدیمی
const PRUNE_DAYS: i64 = 7;

/// یک hit پارس‌شده از یک خط kmsg
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHit {
    pub rule_id: Uuid,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub proto: Option<String>,
    pub dport: Option<u16>,
}

/// کلید تجمیع — همه‌ی hit های همسان در یک پنجره یک ردیف می‌شوند
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HitKey {
    rule_id: Uuid,
    src: String,
    proto: String,
    dport: Option<u16>,
}

#[derive(Debug)]
struct HitAgg {
    hits: u64,
    dst: Option<String>,
}

/// پارس یک رکورد kmsg:
///
/// ```text
/// 16,512,123456789,-;dezh:3f0a1b2c-...: IN=eth0 OUT= SRC=1.2.3.4 DST=5.6.7.8 PROTO=TCP SPT=54321 DPT=22
/// ```
///
/// فقط خطوطی که با `dezh:<uuid>: ` شروع شوند قبول می‌شوند.
pub fn parse_kmsg_line(line: &str) -> Option<ParsedHit> {
    let line = line.trim();
    let (_meta, msg) = line.split_once(';')?;
    let rest = msg.trim_start().strip_prefix("dezh:")?;
    let (uuid_str, fields) = rest.split_once(": ")?;
    let rule_id = Uuid::parse_str(uuid_str.trim()).ok()?;

    let mut src = None;
    let mut dst = None;
    let mut proto = None;
    let mut dport = None;

    for tok in fields.split_whitespace() {
        if let Some(v) = tok.strip_prefix("SRC=") {
            src = Some(v.to_string());
        } else if let Some(v) = tok.strip_prefix("DST=") {
            dst = Some(v.to_string());
        } else if let Some(v) = tok.strip_prefix("PROTO=") {
            proto = Some(v.to_lowercase());
        } else if let Some(v) = tok.strip_prefix("DPT=") {
            dport = v.parse().ok();
        }
        // SPT / IN / OUT / ... فعلاً لازم نیستند
    }

    Some(ParsedHit {
        rule_id,
        src,
        dst,
        proto,
        dport,
    })
}

/// راه‌اندازی کالکتور در یک thread جدا — از `FirewallService::attach_pool`
pub fn spawn_collector(pool: DbPool) {
    let spawned = std::thread::Builder::new()
        .name("dezh-kmsg".into())
        .spawn(move || run(pool));

    if spawned.is_err() {
        eprintln!("[events] failed to spawn kmsg collector thread");
    }
}

fn open_kmsg() -> Option<File> {
    // O_NONBLOCK (0o4000 روی Linux) — حلقه‌ی ما با sleep کار می‌کند نه epoll
    match OpenOptions::new()
        .read(true)
        .custom_flags(0o4000)
        .open("/dev/kmsg")
    {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!(
                "[events] /dev/kmsg unavailable ({e}) — rule-hit events disabled (login events still work)"
            );
            None
        }
    }
}

fn run(pool: DbPool) {
    let Some(mut file) = open_kmsg() else {
        return;
    };

    // پرش از روی تاریخچه‌ی بافر هسته — فقط پیام‌های «از الان» مهم‌اند
    let mut sink = [0u8; 8192];
    loop {
        match file.read(&mut sink) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    eprintln!(
        "[events] kmsg collector started — rule hits (log=true) → security_events every {}s",
        FLUSH_INTERVAL.as_secs()
    );

    let mut agg: HashMap<HitKey, HitAgg> = HashMap::new();
    let mut buf = [0u8; 8192];
    let mut last_flush = Instant::now();
    let mut last_prune = Instant::now();
    let mut dropped: u64 = 0;

    loop {
        // ── ۱) خواندن هرچه آماده است
        loop {
            match file.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let line = String::from_utf8_lossy(&buf[..n]);
                    let Some(hit) = parse_kmsg_line(&line) else {
                        continue;
                    };

                    let key = HitKey {
                        rule_id: hit.rule_id,
                        src: hit.src.clone().unwrap_or_else(|| "unknown".into()),
                        proto: hit.proto.clone().unwrap_or_else(|| "any".into()),
                        dport: hit.dport,
                    };

                    if agg.contains_key(&key) || agg.len() < MAX_KEYS {
                        let dst = hit.dst.clone();
                        let entry = agg.entry(key).or_insert_with(|| HitAgg {
                            hits: 0,
                            dst,
                        });
                        entry.hits += 1;
                        if entry.dst.is_none() {
                            entry.dst = hit.dst;
                        }
                    } else {
                        // سرریز ظرفیت — به‌جای ذخیره، بشمار که در لاگ سرور دیده شود
                        dropped += 1;
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }

        // ── ۲) flush دوره‌ای
        if last_flush.elapsed() >= FLUSH_INTERVAL {
            last_flush = Instant::now();

            if !agg.is_empty() {
                let mut all: Vec<(HitKey, HitAgg)> = agg.drain().collect();
                // مهم‌ترین‌ها (بیشترین hit) اول
                all.sort_by(|a, b| b.1.hits.cmp(&a.1.hits));

                if all.len() > MAX_ROWS_PER_FLUSH {
                    for (_, a) in all[MAX_ROWS_PER_FLUSH..].iter() {
                        dropped += a.hits;
                    }
                    all.truncate(MAX_ROWS_PER_FLUSH);
                }

                let events: Vec<NewSecurityEvent> = all
                    .iter()
                    .map(|(k, v)| NewSecurityEvent {
                        kind: "rule_hit".into(),
                        severity: "warning".into(),
                        source_ip: Some(k.src.clone()),
                        dest_port: k.dport,
                        rule_id: Some(k.rule_id),
                        hits: v.hits,
                        message: format!(
                            "{} packet hit(s) matched rule {} ({}{})",
                            v.hits,
                            k.rule_id,
                            k.proto,
                            k.dport
                                .map(|p| format!(" dport {p}"))
                                .unwrap_or_default()
                        ),
                    })
                    .collect();

                let repo = EventsRepo::new(pool.inner());
                if let Err(e) = block_on(repo.insert_many(&events)) {
                    eprintln!("[events] insert_many failed: {e}");
                }
            }

            if dropped > 0 {
                eprintln!("[events] {dropped} hit(s) dropped (overflow protection)");
                dropped = 0;
            }
        }

        // ── ۳) پاکسازی روزانه‌ی قدیمی‌ها
        if last_prune.elapsed() >= Duration::from_secs(3600) {
            last_prune = Instant::now();
            let repo = EventsRepo::new(pool.inner());
            match block_on(repo.prune_older_than_days(PRUNE_DAYS)) {
                Ok(n) if n > 0 => eprintln!("[events] pruned {n} old event(s)"),
                Ok(_) => {}
                Err(e) => eprintln!("[events] prune failed: {e}"),
            }
        }

        std::thread::sleep(Duration::from_millis(300));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: &str = "3f0a1b2c-4d5e-6f70-8a9b-0c1d2e3f4a5b";

    #[test]
    fn parse_full_nft_log_line() {
        let line = format!(
            "16,512,123456789,-;dezh:{UUID}: IN=eth0 OUT= SRC=203.0.113.7 DST=192.168.1.106 PROTO=TCP SPT=54321 DPT=22"
        );
        let hit = parse_kmsg_line(&line).expect("should parse");

        assert_eq!(hit.rule_id.to_string(), UUID);
        assert_eq!(hit.src.as_deref(), Some("203.0.113.7"));
        assert_eq!(hit.dst.as_deref(), Some("192.168.1.106"));
        assert_eq!(hit.proto.as_deref(), Some("tcp"));
        assert_eq!(hit.dport, Some(22));
    }

    #[test]
    fn parse_udp_line() {
        let line = format!("4,7,999,-;dezh:{UUID}: SRC=10.0.0.5 DST=10.0.0.1 PROTO=UDP SPT=53 DPT=5353");
        let hit = parse_kmsg_line(&line).expect("should parse");
        assert_eq!(hit.proto.as_deref(), Some("udp"));
        assert_eq!(hit.dport, Some(5353));
    }

    #[test]
    fn reject_non_dezh_lines() {
        assert!(parse_kmsg_line("16,1,1,-;random kernel message here").is_none());
        assert!(parse_kmsg_line("16,1,1,-;dezh:not-a-uuid: SRC=1.2.3.4").is_none());
        assert!(parse_kmsg_line("16,1,1,-;dezh: SRC=1.2.3.4").is_none());
    }

    #[test]
    fn parse_line_without_dport() {
        // icmp هیچ DPT ندارد
        let line = format!("16,3,5,-;dezh:{UUID}: SRC=8.8.8.8 DST=192.168.1.1 PROTO=ICMP");
        let hit = parse_kmsg_line(&line).expect("should parse");
        assert_eq!(hit.proto.as_deref(), Some("icmp"));
        assert_eq!(hit.dport, None);
        assert_eq!(hit.src.as_deref(), Some("8.8.8.8"));
    }
}