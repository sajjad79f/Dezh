use std::process::Command;
use std::thread;
use std::time::Duration;

use serde::Serialize;

#[derive(Serialize)]
struct UserActive<'a> {
    username: &'a str,
    hostname: Option<&'a str>,
    ip_address: Option<&'a str>,
    os: Option<&'a str>,
}

#[derive(Serialize)]
struct UserInactive<'a> {
    username: &'a str,
    bytes_in: i64,
    bytes_out: i64,
}

fn main() {
    let base = std::env::var("DEZH_URL").unwrap_or_else(|_| "http://127.0.0.1:7878".into());
    let token = std::env::var("DEZH_TOKEN").expect("set DEZH_TOKEN");
    let interval_secs: u64 = std::env::var("DEZH_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let username = current_user();
    let hostname = hostname();

    eprintln!("[linux-agent] tracking user={username} host={hostname:?}");
    eprintln!("[linux-agent] server={base} interval={interval_secs}s");

    let mut was_active = false;

    loop {
        let is_present = user_is_logged_in(&username);

        if is_present && !was_active {
            let ip = primary_ip();

            eprintln!("[linux-agent] user became active");

            post_json(
                &base,
                &token,
                "/api/agent/user-active",
                &UserActive {
                    username: &username,
                    hostname: hostname.as_deref(),
                    ip_address: ip.as_deref(),
                    os: Some("linux"),
                },
            );

            was_active = true;
        } else if !is_present && was_active {
            eprintln!("[linux-agent] user became inactive");

            post_json(
                &base,
                &token,
                "/api/agent/user-inactive",
                &UserInactive {
                    username: &username,
                    bytes_in: 0,
                    bytes_out: 0,
                },
            );

            was_active = false;
        }

        thread::sleep(Duration::from_secs(interval_secs));
    }
}

fn post_json<T: Serialize>(base: &str, token: &str, path: &str, body: &T) {
    let json = serde_json::to_string(body).expect("json");
    let url = format!("{base}{path}");

    let output = Command::new("curl")
        .args([
            "-s", "-w", "\n%{http_code}", "-X", "POST", &url,
            "-H", &format!("Authorization: Bearer {token}"),
            "-H", "Content-Type: application/json",
            "-d", &json,
        ])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            eprintln!("[linux-agent] {path} -> {}", text.trim());
        }
        Err(e) => eprintln!("[linux-agent] curl error calling {path}: {e}"),
    }
}

fn current_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".into())
}

fn hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn primary_ip() -> Option<String> {
    let out = Command::new("hostname").arg("-I").output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace()
        .find(|p| p.contains('.') && !p.starts_with("127."))
        .map(|s| s.to_string())
}

/// True اگر username الان یه login session داشته باشه (طبق `who`).
/// Session های غیرتعاملی رو تشخیص نمی‌ده -- محدودیت شناخته‌شده‌ست.
fn user_is_logged_in(username: &str) -> bool {
    let Ok(out) = Command::new("who").output() else {
        return false;
    };

    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.split_whitespace().next() == Some(username))
}