//! اعتبارسنجی ورودی‌های سمت سرور
//!
//! چرا سمت سرور؟ چون فرانت قابل دورزدن است. مقادیر source/destination و
//! gateway و interface مستقیماً به nftables و `ip` پاس داده می‌شوند؛
//! این ماژول تضمین می‌کند فقط IP/CIDR/نام معتبر از فیلتر رد شود
//! (دفاع در عمق در برابر تزریق توکن nft و مقادیر آشغال).

use std::net::IpAddr;

// ─── نام و متن ─────────────────────────────────────────────

/// نام rule/NAT: غیرخالی، ≤128 بایت، بدون کاراکتر کنترلی
pub fn check_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name is required".into());
    }
    if name.len() > 128 {
        return Err("name too long (max 128 bytes)".into());
    }
    if name.chars().any(|c| c.is_control()) {
        return Err("name contains control characters".into());
    }
    Ok(name)
}

/// توضیحات: ≤512 بایت، بدون کاراکتر کنترلی
pub fn check_description(desc: &str) -> Result<&str, String> {
    let desc = desc.trim();
    if desc.len() > 512 {
        return Err("description too long (max 512 bytes)".into());
    }
    if desc.chars().any(|c| c.is_control()) {
        return Err("description contains control characters".into());
    }
    Ok(desc)
}

// ─── شبکه ──────────────────────────────────────────────────

/// IP یا CIDR یا "any" — برای source/destination قوانین فایروال و NAT
///
/// مثال‌های معتبر: "any" ، "" ، "10.0.0.5" ، "10.0.0.0/8" ، "::1" ، "fd00::/8"
pub fn check_ip_or_cidr(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok("any".into());
    }
    if value.eq_ignore_ascii_case("any") {
        return Ok("any".into());
    }

    let (addr_part, prefix) = match value.split_once('/') {
        Some((a, p)) => {
            let n: u8 = p
                .parse()
                .map_err(|_| format!("invalid CIDR prefix in `{value}`"))?;
            (a, Some(n))
        }
        None => (value, None),
    };

    let ip: IpAddr = addr_part
        .parse()
        .map_err(|_| format!("invalid IP address: `{addr_part}`"))?;

    if let Some(n) = prefix {
        let max = match ip {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        if n > max {
            return Err(format!("CIDR prefix /{n} out of range (max /{max})"));
        }
    }

    Ok(value.to_string())
}

/// IP خالص — برای gateway و target
pub fn check_ip(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("IP address is required".into());
    }
    value
        .parse::<IpAddr>()
        .map(|_| value.to_string())
        .map_err(|_| format!("invalid IP address: `{value}`"))
}

/// CIDR اجباری (یا IP خالص که به CIDR تبدیل می‌شود) — برای مقصد روت‌ها
pub fn check_cidr(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("destination is required".into());
    }
    if value.contains('/') {
        return check_ip_or_cidr(value);
    }
    // IP خالص را به CIDR کامل تبدیل کن
    let ip: IpAddr = value
        .parse()
        .map_err(|_| format!("invalid destination: `{value}`"))?;
    Ok(match ip {
        IpAddr::V4(v4) => format!("{v4}/32"),
        IpAddr::V6(v6) => format!("{v6}/128"),
    })
}

/// نام interface لینوکس (IFNAMSIZ-1 = 15 بایت)
pub fn check_iface_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("interface name is required".into());
    }
    if name.len() > 15 {
        return Err("interface name too long (max 15 bytes)".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@' | ':'))
    {
        return Err(format!(
            "invalid interface name `{name}` (allowed: a-z 0-9 . - _ @ :)"
        ));
    }
    Ok(name)
}

// ─── پورت و اولویت ─────────────────────────────────────────

/// پورت باید ۱ تا ۶۵۵۳۵ باشد (0 غیرمجاز)
pub fn check_port(port: Option<u16>) -> Result<Option<u16>, String> {
    match port {
        Some(0) => Err("port must be 1-65535".into()),
        other => Ok(other),
    }
}

/// اولویت rule: 0 تا 100000
pub fn check_priority(priority: u32) -> Result<u32, String> {
    if priority > 100_000 {
        return Err("priority too high (max 100000)".into());
    }
    Ok(priority)
}

// ─── Zone و Interface Config ───────────────────────────────

/// نام zone: 2..=32 کاراکتر، حروف/عدد/خط تیره/زیرخط
pub fn check_zone_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.len() < 2 || name.len() > 32 {
        return Err("zone name must be 2-32 characters".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(format!(
            "invalid zone name `{name}` (allowed: a-z 0-9 - _)"
        ));
    }
    Ok(name)
}

/// حالت IPv4 interface
pub fn check_ipv4_mode(mode: &str) -> Result<&str, String> {
    let mode = mode.trim();
    match mode {
        "none" | "static" | "dhcp" => Ok(mode),
        other => Err(format!(
            "invalid ipv4_mode `{other}` (allowed: none | static | dhcp)"
        )),
    }
}

/// کانفیگ کامل interface: static → آدرس و gateway اجباری و معتبر
pub fn check_interface_config(
    ipv4_mode: &str,
    address_cidr: Option<&str>,
    gateway: Option<&str>,
) -> Result<(), String> {
    match ipv4_mode {
        "static" => {
            let cidr = address_cidr
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "address_cidr is required for static mode".to_string())?;
            check_ip_or_cidr(cidr)?;
            let gw = gateway
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "gateway is required for static mode".to_string())?;
            check_ip(&gw)?;
            Ok(())
        }
        "dhcp" | "none" => Ok(()),
        other => check_ipv4_mode(other).map(|_| ()),
    }
}

// ─── کاربر و رمز عبور ──────────────────────────────────────

/// نام کاربری: 3..=32، شروع با حرف/عدد، مجاز: a-z 0-9 . _ -
pub fn check_username(username: &str) -> Result<&str, String> {
    let username = username.trim();
    if username.len() < 3 || username.len() > 32 {
        return Err("username must be 3-32 characters".into());
    }
    let first = username.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Err("username must start with a letter or digit".into());
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(format!(
            "invalid username `{username}` (allowed: a-z 0-9 . _ -)"
        ));
    }
    Ok(username)
}

/// سیاست رمز عبور: 8..=128 کاراکتر، حداقل یک حرف و یک عدد
pub fn check_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }
    if password.len() > 128 {
        return Err("password too long (max 128 characters)".into());
    }
    if !password.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err("password must contain at least one letter".into());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("password must contain at least one digit".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_or_cidr_accepts_valid() {
        assert_eq!(check_ip_or_cidr("any").unwrap(), "any");
        assert_eq!(check_ip_or_cidr("").unwrap(), "any");
        assert_eq!(check_ip_or_cidr("  ").unwrap(), "any");
        assert_eq!(check_ip_or_cidr("10.0.0.5").unwrap(), "10.0.0.5");
        assert_eq!(check_ip_or_cidr("10.0.0.0/8").unwrap(), "10.0.0.0/8");
        assert_eq!(check_ip_or_cidr("fd00::/8").unwrap(), "fd00::/8");
        assert_eq!(check_ip_or_cidr("ANY").unwrap(), "any");
    }

    #[test]
    fn ip_or_cidr_rejects_injection_and_garbage() {
        // تزریق توکن nft — باید رد شود
        assert!(check_ip_or_cidr("10.0.0.1 comment \"x\"").is_err());
        assert!(check_ip_or_cidr("10.0.0.1; drop").is_err());
        assert!(check_ip_or_cidr("999.1.1.1").is_err());
        assert!(check_ip_or_cidr("10.0.0.0/33").is_err());
        assert!(check_ip_or_cidr("10.0.0.0/-1").is_err());
        assert!(check_ip_or_cidr("10.0.0.0/eight").is_err());
    }

    #[test]
    fn cidr_converts_plain_ip() {
        assert_eq!(check_cidr("10.0.0.5").unwrap(), "10.0.0.5/32");
        assert_eq!(check_cidr("fd00::5").unwrap(), "fd00::5/128");
        assert_eq!(check_cidr("10.0.0.0/8").unwrap(), "10.0.0.0/8");
        assert!(check_cidr("nope").is_err());
        assert!(check_cidr("").is_err());
    }

    #[test]
    fn iface_name_rules() {
        assert!(check_iface_name("eth0").is_ok());
        assert!(check_iface_name("enp3s0").is_ok());
        assert!(check_iface_name("br-lan").is_ok());
        assert!(check_iface_name("").is_err());
        assert!(check_iface_name("eth0; drop").is_err());
        assert!(check_iface_name("verylonginterfacename16").is_err());
    }

    #[test]
    fn password_policy() {
        assert!(check_password("Admin123!").is_ok());
        assert!(check_password("short1").is_err()); // <8
        assert!(check_password("nodigitshere").is_err());
        assert!(check_password("12345678").is_err());
    }

    #[test]
    fn username_rules() {
        assert!(check_username("admin").is_ok());
        assert!(check_username("op_1").is_ok());
        assert!(check_username("ab").is_err());
        assert!(check_username("_start").is_err());
        assert!(check_username("bad name").is_err());
    }

    #[test]
    fn interface_config_rules() {
        assert!(check_interface_config("static", Some("192.168.1.10/24"), Some("192.168.1.1")).is_ok());
        assert!(check_interface_config("static", None, Some("192.168.1.1")).is_err());
        assert!(check_interface_config("static", Some("bad"), Some("192.168.1.1")).is_err());
        assert!(check_interface_config("dhcp", None, None).is_ok());
        assert!(check_interface_config("none", None, None).is_ok());
        assert!(check_interface_config("bridge", None, None).is_err());
    }

    #[test]
    fn port_and_priority() {
        assert!(check_port(Some(0)).is_err());
        assert!(check_port(Some(443)).is_ok());
        assert!(check_port(None).is_ok());
        assert!(check_priority(100_001).is_err());
        assert!(check_priority(100).is_ok());
    }
}