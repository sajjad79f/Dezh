use crate::error::{NetworkError, NetworkResult};

/// (name, is_up, addresses) — بدون lo
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