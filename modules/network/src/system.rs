use crate::error::{NetworkError, NetworkResult};

/// (name, is_up, addresses) — بدون lo
/// (rx_bytes, tx_bytes) از sysfs — اگر موجود نبود None (N1: خلاصهٔ داشبورد)
pub fn read_iface_stats(name: &str) -> Option<(u64, u64)> {
    // نام اینترفیس سختگیرانه اعتبارسنجی میشود (در مسیر sysfs جاسازی میشود)
    if name.is_empty()
        || name.len() > 15
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '@' || c == '-' || c == '_')
    {
        return None;
    }
    let rx = std::fs::read_to_string(format!("/sys/class/net/{name}/statistics/rx_bytes")).ok()?;
    let tx = std::fs::read_to_string(format!("/sys/class/net/{name}/statistics/tx_bytes")).ok()?;
    let rx: u64 = rx.trim().parse().ok()?;
    let tx: u64 = tx.trim().parse().ok()?;
    Some((rx, tx))
}

pub fn detect_system_interfaces() -> NetworkResult<Vec<(String, bool, Vec<String>)>> {
    let output = std::process::Command::new("ip")
        .args(["-o", "-4", "addr", "show"])
        .output()
        .map_err(|e| NetworkError::Internal(format!("ip failed: {e}")))?;

    if !output.status.success() {
        return Err(NetworkError::Internal(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut map: std::collections::BTreeMap<String, (bool, Vec<String>)> =
        std::collections::BTreeMap::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let name = parts[1].trim_end_matches(':').to_string();
        if name == "lo" {
            continue;
        }
        let addr = parts
            .iter()
            .position(|p| *p == "inet")
            .and_then(|i| parts.get(i + 1))
            .map(|s| s.to_string());
        let entry = map.entry(name).or_insert((true, Vec::new()));
        if let Some(a) = addr {
            entry.1.push(a);
        }
    }

    let link = std::process::Command::new("ip")
        .args(["-o", "link", "show"])
        .output()
        .map_err(|e| NetworkError::Internal(format!("ip link failed: {e}")))?;

    if link.status.success() {
        let text = String::from_utf8_lossy(&link.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let name = parts[1].trim_end_matches(':').to_string();
            if name == "lo" {
                continue;
            }
            let up = line.contains("state UP");
            map.entry(name).or_insert((up, Vec::new())).0 = up;
        }
    }

    Ok(map
        .into_iter()
        .map(|(name, (up, addresses))| (name, up, addresses))
        .collect())
}

pub fn run_ip(args: &[&str]) -> NetworkResult<String> {
    if let Ok(output) = std::process::Command::new("sudo")
        .arg("-n")
        .arg("ip")
        .args(args)
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.contains("RTNETLINK") || err.contains("Cannot") || err.contains("Error") {
            // ممکن است sudo کار کرده و خود ip خطا داده
            if !err.contains("password") && !err.contains("a terminal is required") {
                return Err(NetworkError::Internal(err));
            }
        }
    }

    let output = std::process::Command::new("ip")
        .args(args)
        .output()
        .map_err(|e| NetworkError::Internal(format!("ip failed: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.contains("Operation not permitted") {
            return Err(NetworkError::Internal(
                "Operation not permitted — run Dezh as root or grant CAP_NET_ADMIN".into(),
            ));
        }
        Err(NetworkError::Internal(err))
    }
}


// ─── [N1b] تغییر نام اینترفیس در سطح کرنل ─────────────────────────

/// وضعیت admin واقعی لینک (پرچم UP داخل <...>) — نه oper state
fn link_admin_up(name: &str) -> NetworkResult<bool> {
    let out = run_ip(&["-o", "link", "show", "dev", name])?;
    let flags = out
        .split('<')
        .nth(1)
        .and_then(|s| s.split('>').next())
        .unwrap_or("");
    Ok(flags.split(',').any(|f| f.trim() == "UP"))
}

/// تغییر نام در سطح کرنل. ترتیب امن:
/// down → rename → بازگرداندن وضعیت admin قبلی.
/// اگر rename شکست خورد، وضعیت قبلی best-effort برمی‌گردد.
/// توجه: چند ثانیه قطعی لینک طبیعی است (کرنل rename را فقط روی لینک down می‌پذیرد).
pub fn rename_interface_link(old: &str, new: &str) -> NetworkResult<()> {
    let was_up = link_admin_up(old)?;

    run_ip(&["link", "set", "dev", old, "down"])?;

    if let Err(e) = run_ip(&["link", "set", "dev", old, "name", new]) {
        // بازگردانی وضعیت قبلی
        let _ = run_ip(&["link", "set", "dev", old, if was_up { "up" } else { "down" }]);
        return Err(e);
    }

    run_ip(&["link", "set", "dev", new, if was_up { "up" } else { "down" }])?;
    Ok(())
}


// ─── [N2] PPPoE — کنترل pppd و نشست‌های زنده ───────────────────────

/// آیا نشست PPPoE با این پروفایل در حال اجراست؟
pub fn pppd_running(profile: &str) -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", &format!("linkname dezh-{profile}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// اجرای pppd برای یک پروفایل (daemonize خودِ pppd).
/// نیازمند بستهٔ ppp با پلاگین rp-pppoe روی سیستم است.
#[allow(clippy::too_many_arguments)]
pub fn start_pppoe(
    profile: &str,
    parent_iface: &str,
    username: &str,
    password: &str,
    mtu: u32,
    default_route: bool,
    keepalive: u32,
) -> NetworkResult<()> {
    if pppd_running(profile) {
        return Err(NetworkError::Internal(format!(
            "pppoe '{profile}' is already running"
        )));
    }

    let args: Vec<String> = vec![
        "linkname".into(),
        format!("dezh-{profile}"),
        "plugin".into(),
        "rp-pppoe.so".into(),
        parent_iface.to_string(),
        "user".into(),
        username.to_string(),
        "password".into(),
        password.to_string(),
        "mtu".into(),
        mtu.to_string(),
        "mru".into(),
        mtu.to_string(),
        "noipdefault".into(),
        if default_route { "defaultroute".into() } else { "nodefaultroute".into() },
        "persist".into(),
        "maxfail".into(),
        "0".into(),
        "holdoff".into(),
        "5".into(),
        "lcp-echo-interval".into(),
        keepalive.to_string(),
        "lcp-echo-failure".into(),
        "3".into(),
    ];

    let mut child = std::process::Command::new("pppd")
        .args(&args)
        .spawn()
        .map_err(|e| NetworkError::Internal(format!("pppd failed to start: {e}")))?;
    // pppd فوراً fork میکند؛ صبر برای خروج پروسهٔ پدر تا zombie نماند
    let _ = child.wait();

    std::thread::sleep(std::time::Duration::from_millis(400));
    if !pppd_running(profile) {
        return Err(NetworkError::Internal(
            "pppd exited immediately — check rp-pppoe plugin, credentials and the parent interface (journalctl -u dezh-dcm / dmesg)"
                .into(),
        ));
    }
    Ok(())
}

/// توقف نشست PPPoE پروفایل داده‌شده (SIGTERM → LCP terminate تمیز)
pub fn stop_pppoe(profile: &str) -> NetworkResult<()> {
    let out = std::process::Command::new("pkill")
        .args(["-TERM", "-f", &format!("linkname dezh-{profile}")])
        .output()
        .map_err(|e| NetworkError::Internal(format!("pkill failed: {e}")))?;
    if !out.status.success() {
        return Err(NetworkError::NotFound(format!(
            "no running pppd session for '{profile}'"
        )));
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    Ok(())
}

/// نشست‌های ppp زندهٔ سیستم — از /run/pppX.pid (یا /var/run) و
/// /proc/<pid>/cmdline صاحب نشست (linkname dezh-<profile>) درمیآید.
pub fn list_ppp_sessions() -> Vec<crate::models::PppoeSession> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for dir in ["/run", "/var/run"] {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname = fname.to_string_lossy();
            if !fname.starts_with("ppp") || !fname.ends_with(".pid") {
                continue;
            }
            let iface = fname.trim_end_matches(".pid").to_string();
            if !seen.insert(iface.clone()) {
                continue;
            }
            let pid: u32 = match std::fs::read_to_string(entry.path())
                .ok()
                .and_then(|s| s.trim().parse().ok())
            {
                Some(p) => p,
                None => continue,
            };

            // صاحب نشست از cmdline پیداست
            // [N3-fix] رشته‌ها owned میشوند (E0515) و الگوی
            // file /etc/ppp/options.dezh-* (نشست L2TP) هم شناسایی میشود
            let profile = std::fs::read(format!("/proc/{pid}/cmdline"))
                .ok()
                .map(|bytes| {
                    bytes
                        .split(|&b| b == 0)
                        .filter_map(|t| std::str::from_utf8(t).ok())
                        .map(|t| t.to_owned())
                        .collect::<Vec<String>>()
                })
                .and_then(|toks| {
                    toks.iter()
                        .position(|t| *t == "linkname")
                        .and_then(|i| toks.get(i + 1))
                        .and_then(|t| t.strip_prefix("dezh-"))
                        .map(|s| s.to_string())
                        .or_else(|| {
                            toks.iter()
                                .position(|t| t.as_str() == "file")
                                .and_then(|i| toks.get(i + 1))
                                .and_then(|t| t.strip_prefix("/etc/ppp/options.dezh-"))
                                .map(|s| s.to_string())
                        })
                });

            let (local_ip, remote_ip) = ppp_addr_pair(&iface);
            out.push(crate::models::PppoeSession {
                iface,
                profile,
                local_ip,
                remote_ip,
                up: true,
            });
        }
    }
    out
}

/// (local, remote) از خروجی `ip -o -4 addr show dev <iface>`
fn ppp_addr_pair(iface: &str) -> (Option<String>, Option<String>) {
    // نام اینترفیس سختگیرانه اعتبارسنجی میشود (در مسیر دستور جاسازی میشود)
    let name_ok = !iface.is_empty()
        && iface.len() <= 15
        && iface
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '@' || c == '-' || c == '_');
    if !name_ok {
        return (None, None);
    }
    let out = match run_ip(&["-o", "-4", "addr", "show", "dev", iface]) {
        Ok(o) => o,
        Err(_) => return (None, None),
    };
    let toks: Vec<&str> = out.split_whitespace().collect();
    let local = toks
        .iter()
        .position(|t| *t == "inet")
        .and_then(|i| toks.get(i + 1))
        .map(|s| s.to_string());
    let remote = toks
        .iter()
        .position(|t| *t == "peer")
        .and_then(|i| toks.get(i + 1))
        .map(|s| s.split('/').next().unwrap_or(s).to_string());
    (local, remote)
}


// ─── [N3] پایدارسازی rename در فایل‌های ifupdown ───────────────────

/// در یک خط ifupdown، توکن برابر old را با new عوض میکند — فقط برای
/// استنزاهای auto / allow-hotplug / iface. سایر خطوط دست‌نخورده.
/// (برای iface فقط توکن دوم یعنی نام اینترفیس عوض میشود)
fn subst_iface_tokens(line: &str, old: &str, new: &str) -> Option<String> {
    let kw = line.split_whitespace().next().unwrap_or("");
    if kw != "auto" && kw != "allow-hotplug" && kw != "iface" {
        return None;
    }
    let hit = line.split_whitespace().enumerate().any(|(i, t)| {
        t == old && (kw != "iface" || i == 1)
    });
    if !hit {
        return None;
    }
    // بازسازی خط با حفظ دقیق فاصله‌ها
    let mut out = String::with_capacity(line.len() + new.len());
    let mut word = String::new();
    for ch in line.chars() {
        if ch.is_whitespace() {
            if word == old {
                out.push_str(new);
            } else {
                out.push_str(&word);
            }
            word.clear();
            out.push(ch);
        } else {
            word.push(ch);
        }
    }
    if word == old {
        out.push_str(new);
    } else {
        out.push_str(&word);
    }
    Some(out)
}

/// نام اینترفیس old را در /etc/network/interfaces و همهٔ فایل‌های
/// /etc/network/interfaces.d/* به new تغییر میدهد. برمیگرداند: تعداد
/// فایل‌های تغییرکرده. فایل نبودن → Ok(0) (سیستم‌های netplan).
/// هر فایل قبل از نوشتن به <name>.dezh-bak بکاپ و اتمیک نوشته میشود.
pub fn update_interfaces_file(old: &str, new: &str) -> NetworkResult<usize> {
    let mut changed = 0usize;

    let mut targets: Vec<std::path::PathBuf> = vec!["/etc/network/interfaces".into()];
    if let Ok(rd) = std::fs::read_dir("/etc/network/interfaces.d") {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                targets.push(p);
            }
        }
    }

    for path in targets {
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let mut touched = false;
        let mut out = String::with_capacity(text.len() + 16);
        for line in text.split_inclusive('\n') {
            let had_nl = line.ends_with('\n');
            let core = line.trim_end_matches('\n');
            match subst_iface_tokens(core, old, new) {
                Some(l) => {
                    out.push_str(&l);
                    if had_nl {
                        out.push('\n');
                    }
                    touched = true;
                }
                None => out.push_str(line),
            }
        }
        if touched {
            let _ = std::fs::copy(&path, path.with_extension("dezh-bak"));
            let tmp = path.with_extension("dezh-tmp");
            std::fs::write(&tmp, out)
                .and_then(|_| std::fs::rename(&tmp, &path))
                .map_err(|e| {
                    NetworkError::Internal(format!("write {}: {e}", path.display()))
                })?;
            changed += 1;
        }
    }
    Ok(changed)
}


// ─── [N3] VPN WAN — PPTP / L2TP ────────────────────────────────────

const L2TP_CONF: &str = "/etc/xl2tpd/xl2tpd.conf";
const L2TP_CTRL: &str = "/var/run/xl2tpd/l2tp-control";

fn systemctl_ok(args: &[&str]) -> bool {
    std::process::Command::new("systemctl")
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn xl2tpd_installed() -> bool {
    systemctl_ok(&["cat", "xl2tpd.service"])
        || std::path::Path::new("/usr/sbin/xl2tpd").exists()
        || std::path::Path::new("/usr/bin/xl2tpd").exists()
}

/// اگر xl2tpd در حال اجراست، کانفیگش را دوباره بخوان (try-restart)
pub fn reload_xl2tpd() {
    if systemctl_ok(&["is-active", "--quiet", "xl2tpd"]) {
        let _ = std::process::Command::new("systemctl")
            .args(["try-restart", "xl2tpd"])
            .output();
    }
}

/// توقف نشست ppp متعلق به یک اتصال VPN — هر دو الگو امتحان میشود:
/// pppd مستقیم (pptp): linkname dezh-<name>
/// pppd از طرف xl2tpd (l2tp): options.dezh-<name> در cmdline
fn stop_ppp_by_patterns(profile: &str) -> NetworkResult<()> {
    for pat in [
        format!("linkname dezh-{profile}"),
        format!("options.dezh-{profile}"),
    ] {
        let out = std::process::Command::new("pkill")
            .args(["-TERM", "-f", &pat])
            .output()
            .map_err(|e| NetworkError::Internal(format!("pkill failed: {e}")))?;
        if out.status.success() {
            std::thread::sleep(std::time::Duration::from_millis(300));
            return Ok(());
        }
    }
    Err(NetworkError::NotFound(format!(
        "no running ppp session for '{profile}'"
    )))
}

/// آیا نشست pppd این اتصال فعاله؟ (pptp: linkname / l2tp: optfile)
fn vpn_session_active(profile: &str, kind: &str) -> bool {
    let pat = if kind == "l2tp" {
        format!("options.dezh-{profile}")
    } else {
        format!("linkname dezh-{profile}")
    };
    std::process::Command::new("pgrep")
        .args(["-f", &pat])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// برقراری اتصال PPTP با pppd + pty pptp (بستهٔ pptp-linux).
/// رشتهٔ pty از طریق sh اجرا میشود؛ server قبلاً در سرویس سختگیرانه
/// validate شده (فقط a-z 0-9 . - _ :) تا تزریق فرمان ممکن نباشد.
#[allow(clippy::too_many_arguments)]
pub fn start_pptp(
    profile: &str,
    server: &str,
    username: &str,
    password: &str,
    mtu: u32,
    default_route: bool,
    keepalive: u32,
) -> NetworkResult<()> {
    if vpn_session_active(profile, "pptp") {
        return Err(NetworkError::Internal(format!(
            "vpn '{profile}' is already running"
        )));
    }

    let args: Vec<String> = vec![
        "linkname".into(),
        format!("dezh-{profile}"),
        "pty".into(),
        format!("pptp {server} --nolaunchpppd"),
        "name".into(),
        username.to_string(),
        "password".into(),
        password.to_string(),
        "noauth".into(),
        "require-mppe-128".into(),
        "mtu".into(),
        mtu.to_string(),
        "mru".into(),
        mtu.to_string(),
        "noipdefault".into(),
        if default_route { "defaultroute".into() } else { "nodefaultroute".into() },
        "persist".into(),
        "maxfail".into(),
        "0".into(),
        "holdoff".into(),
        "5".into(),
        "lcp-echo-interval".into(),
        keepalive.to_string(),
        "lcp-echo-failure".into(),
        "3".into(),
    ];

    let mut child = std::process::Command::new("pppd")
        .args(&args)
        .spawn()
        .map_err(|e| NetworkError::Internal(format!("pppd failed to start: {e}")))?;
    // pppd فوراً fork میکند؛ صبر برای خروج پروسهٔ پدر تا zombie نماند
    let _ = child.wait();

    std::thread::sleep(std::time::Duration::from_millis(500));
    if !vpn_session_active(profile, "pptp") {
        return Err(NetworkError::Internal(
            "pppd exited immediately — check pptp-linux package, credentials, MPPE and server reachability (journalctl -u dezh-dcm)"
                .into(),
        ));
    }
    Ok(())
}

/// نوشتن فایل options مخصوص هر اتصال L2TP (pppd آن را از طرف
/// xl2tpd میخواند؛ مسیرش در [lac] با ppp optfile معرفی میشود)
pub fn write_l2tp_options(
    profile: &str,
    username: &str,
    password: &str,
    mtu: u32,
    default_route: bool,
    keepalive: u32,
) -> NetworkResult<()> {
    let path = format!("/etc/ppp/options.dezh-{profile}");
    let dr = if default_route { "defaultroute" } else { "nodefaultroute" };
    let body = format!(
        "noauth\nname {username}\npassword {password}\nnoipdefault\n{dr}\nmtu {mtu}\nmru {mtu}\npersist\nmaxfail 0\nlcp-echo-interval {keepalive}\nlcp-echo-failure 3\n"
    );
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, body)
        .map_err(|e| NetworkError::Internal(format!("write {path}: {e}")))?;
    Ok(())
}

/// بازسازی بخش مدیریت‌شدهٔ دژ در xl2tpd.conf از روی ردیف‌های l2tp.
/// بخش بین مارکرها هر بار از نو نوشته میشود؛ بقیهٔ فایل دست‌نخورده.
pub fn regenerate_l2tp_conf(rows: &[(String, String)]) -> NetworkResult<()> {
    let begin = "# >>> dezh managed (do not edit) >>>";
    let end = "# <<< dezh managed <<<";
    let text = std::fs::read_to_string(L2TP_CONF).unwrap_or_default();

    let (base, tail) = match text.find(begin) {
        Some(i) => {
            let head = &text[..i];
            let rest = &text[i..];
            match rest.find(end) {
                Some(j) => {
                    let after = &rest[j + end.len()..];
                    (head.to_string(), after.trim_start_matches('\n').to_string())
                }
                // بخش ناقص — دور ریخته میشود
                None => (head.to_string(), String::new()),
            }
        }
        None => (text.clone(), String::new()),
    };

    let mut out = String::with_capacity(text.len() + 256);
    out.push_str(&base);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(begin);
    out.push('\n');
    for (name, server) in rows {
        out.push_str(&format!(
            "\n[lac dezh-{name}]\nlns = {server}\nppp optfile = /etc/ppp/options.dezh-{name}\nredial = yes\nredial timeout = 15\nmax redials = 0\n"
        ));
    }
    out.push_str(end);
    out.push('\n');
    if !tail.is_empty() {
        out.push('\n');
        out.push_str(&tail);
    }

    if let Some(parent) = std::path::Path::new(L2TP_CONF).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(L2TP_CONF, out)
        .map_err(|e| NetworkError::Internal(format!("write {L2TP_CONF}: {e}")))?;
    Ok(())
}

/// اتصال L2TP: اطمینان از روشن‌بودن xl2tpd + فرمان c به سوکت کنترل.
/// فایل‌های کانفیگ هنگام save نوشته شدهاند؛ اینجا فقط dial میکنیم.
pub fn start_l2tp(profile: &str) -> NetworkResult<()> {
    if vpn_session_active(profile, "l2tp") {
        return Err(NetworkError::Internal(format!(
            "vpn '{profile}' is already running"
        )));
    }

    if !systemctl_ok(&["is-active", "--quiet", "xl2tpd"]) {
        if !xl2tpd_installed() {
            return Err(NetworkError::Internal(
                "xl2tpd is not installed — run: sudo apt install xl2tpd".into(),
            ));
        }
        if !systemctl_ok(&["start", "xl2tpd"]) {
            return Err(NetworkError::Internal(
                "could not start xl2tpd.service — check 'systemctl status xl2tpd'".into(),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(600));
    }

    std::fs::create_dir_all("/var/run/xl2tpd").ok();
    std::fs::write(L2TP_CTRL, format!("c dezh-{profile}\n")).map_err(|e| {
        NetworkError::Internal(format!(
            "cannot write xl2tpd control socket ({L2TP_CTRL}): {e}"
        ))
    })?;

    // انتظار تا ۸ ثانیه برای بالا آمدن نشست
    let mut up = false;
    for _ in 0..16 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if vpn_session_active(profile, "l2tp") {
            up = true;
            break;
        }
    }
    if !up {
        return Err(NetworkError::Internal(
            "L2TP tunnel did not come up within 8s — check server, credentials and 'journalctl -u xl2tpd'"
                .into(),
        ));
    }
    Ok(())
}

/// قطع اتصال VPN (PPTP یا L2TP)
pub fn stop_vpn(profile: &str, kind: &str) -> NetworkResult<()> {
    if kind == "l2tp" {
        let _ = std::fs::write(L2TP_CTRL, format!("d dezh-{profile}\n"));
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    stop_ppp_by_patterns(profile)
}
