use std::process::Command;

use crate::errors::FirewallError;

/// Adapter نازک دور nftables (`nft`) -- خودش پکت فیلترینگ رو بازسازی
/// نمی‌کنه، فقط همون ابزار موجود رو صدا می‌زنه.
/// فرض می‌کند base ruleset (`table inet filter` با `chain input`) از
/// قبل روی ایمیج ساخته شده.
pub struct FirewallService;

impl FirewallService {
    pub fn new() -> Self {
        Self
    }

    fn run_nft(args: &[&str]) -> Result<String, FirewallError> {
        let output = Command::new("nft")
            .args(args)
            .output()
            .map_err(|_| FirewallError::NftNotFound)?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(FirewallError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    }

    pub fn list_rules(&self) -> Result<String, FirewallError> {
        Self::run_nft(&["list", "ruleset"])
    }

    pub fn block_ip(&self, ip: &str) -> Result<(), FirewallError> {
        Self::run_nft(&[
            "add", "rule", "inet", "filter", "input",
            "ip", "saddr", ip, "drop",
        ])?;

        Ok(())
    }

    /// nftables حذف "بر اساس تطبیق" ندارد -- اول باید handle رول رو با
    /// `-a` پیدا کرد، بعد با همون handle حذفش کرد. پارس‌کردن خط خروجی
    /// اینجا Best-effort است (یه Parser واقعی nft نیست) و اگه فرمت
    /// خروجی nft عوض بشه ممکنه بشکنه.
    pub fn unblock_ip(&self, ip: &str) -> Result<(), FirewallError> {
        let listing = Self::run_nft(&["-a", "list", "chain", "inet", "filter", "input"])?;

        let handle = listing
            .lines()
            .find(|line| line.contains(ip) && line.contains("drop"))
            .and_then(|line| line.rsplit("handle").next())
            .and_then(|tail| tail.trim().split_whitespace().next())
            .ok_or_else(|| {
                FirewallError::CommandFailed(format!("no matching rule found for {ip}"))
            })?;

        Self::run_nft(&["delete", "rule", "inet", "filter", "input", "handle", handle])?;

        Ok(())
    }
}