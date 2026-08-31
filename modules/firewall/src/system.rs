use crate::error::{FirewallError, FirewallResult};

/// Returns (name, is_up, addresses) for each non-loopback interface.
pub fn detect_system_interfaces() -> FirewallResult<Vec<(String, bool, Vec<String>)>> {
    let output = std::process::Command::new("ip")
        .args(["-o", "-4", "addr", "show"])
        .output()
        .map_err(|e| FirewallError::Internal(format!("failed to run ip: {e}")))?;

    if !output.status.success() {
        return Err(FirewallError::Internal(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut map: std::collections::BTreeMap<String, (bool, Vec<String>)> =
        std::collections::BTreeMap::new();

    for line in text.lines() {
        // example: 2: eth0    inet 192.168.1.10/24 ...
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

    // Also pick up interfaces that are down (no addr line) via `ip -o link`
    let link = std::process::Command::new("ip")
        .args(["-o", "link", "show"])
        .output()
        .map_err(|e| FirewallError::Internal(format!("failed to run ip link: {e}")))?;

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