use std::process::Command;

use crate::error::{RoutingError, RoutingResult};
use crate::models::RouteEntry;

pub struct RoutingService;

impl RoutingService {
    pub fn new() -> Self {
        Self
    }

    /// اول `sudo -n ip` (بدون پسورد)، اگر نشد خود `ip`.
    /// برای NGFW روی Debian معمولاً باید dcm با root اجرا شود
    /// یا در sudoers خط NOPASSWD برای /sbin/ip تعریف شود.
    fn run_ip(args: &[&str]) -> RoutingResult<String> {
        // try: sudo -n ip ...
        let sudo = Command::new("sudo")
            .arg("-n")
            .arg("ip")
            .args(args)
            .output();

        if let Ok(output) = sudo {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
            }
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            // اگر sudo اصلاً اجازه نداشت، برو سراغ ip مستقیم
            if !err.contains("password is required")
                && !err.contains("a password is required")
                && !err.contains("not allowed")
                && !err.is_empty()
                && output.status.code() != Some(1)
            {
                // خطای واقعی از ip (مثلاً route تکراری)
                if err.contains("RTNETLINK") || err.contains("File exists") || err.contains("No such")
                {
                    return Err(RoutingError::Internal(err));
                }
            }
        }

        let output = Command::new("ip")
            .args(args)
            .output()
            .map_err(|e| RoutingError::Internal(format!("ip failed: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if err.contains("Operation not permitted") {
                return Err(RoutingError::Internal(
                    "Operation not permitted — Dezh must run as root (or grant CAP_NET_ADMIN / sudo NOPASSWD for /sbin/ip)".into(),
                ));
            }
            Err(RoutingError::Internal(err))
        }
    }

    pub fn list_routes(&self) -> RoutingResult<Vec<RouteEntry>> {
        let out = Self::run_ip(&["-4", "route", "show"])?;
        let mut routes = Vec::new();

        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let destination = parts[0].to_string();
            let mut gateway = None;
            let mut device = None;
            let mut proto = None;
            let mut metric = None;

            let mut i = 1;
            while i < parts.len() {
                match parts[i] {
                    "via" if i + 1 < parts.len() => {
                        gateway = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "dev" if i + 1 < parts.len() => {
                        device = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "proto" if i + 1 < parts.len() => {
                        proto = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "metric" if i + 1 < parts.len() => {
                        metric = parts[i + 1].parse().ok();
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            routes.push(RouteEntry {
                destination,
                gateway,
                device,
                proto,
                metric,
            });
        }

        Ok(routes)
    }

    pub fn add_route(
        &self,
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        if destination.trim().is_empty() {
            return Err(RoutingError::Invalid("destination required".into()));
        }
        if gateway.is_none() && device.is_none() {
            return Err(RoutingError::Invalid(
                "at least gateway or device is required".into(),
            ));
        }

        let mut args = vec!["route", "add", destination];
        if let Some(gw) = gateway {
            if !gw.is_empty() {
                args.push("via");
                args.push(gw);
            }
        }
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }

        Self::run_ip(&args)?;
        Ok(())
    }

    pub fn delete_route(
        &self,
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        let mut args = vec!["route", "del", destination];
        if let Some(gw) = gateway {
            if !gw.is_empty() {
                args.push("via");
                args.push(gw);
            }
        }
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }
        Self::run_ip(&args)?;
        Ok(())
    }

    pub fn set_default_gateway(&self, gateway: &str, device: Option<&str>) -> RoutingResult<()> {
        let _ = Self::run_ip(&["route", "del", "default"]);

        let mut args = vec!["route", "add", "default", "via", gateway];
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }
        Self::run_ip(&args)?;
        Ok(())
    }
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RoutingService {
    fn clone(&self) -> Self {
        Self
    }
}