use std::process::Command;

use crate::error::{RoutingError, RoutingResult};
use crate::models::RouteEntry;

pub struct RoutingService;

impl RoutingService {
    pub fn new() -> Self {
        Self
    }

    fn run_ip(args: &[&str]) -> RoutingResult<String> {
        let output = Command::new("ip")
            .args(args)
            .output()
            .map_err(|e| RoutingError::Internal(format!("ip failed: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(RoutingError::Internal(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
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

        let mut args = vec!["route", "add", destination];
        if let Some(gw) = gateway {
            args.push("via");
            args.push(gw);
        }
        if let Some(dev) = device {
            args.push("dev");
            args.push(dev);
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
            args.push("via");
            args.push(gw);
        }
        if let Some(dev) = device {
            args.push("dev");
            args.push(dev);
        }
        Self::run_ip(&args)?;
        Ok(())
    }

    pub fn set_default_gateway(&self, gateway: &str, device: Option<&str>) -> RoutingResult<()> {
        // حذف default قبلی (اگر بود) — خطا را نادیده بگیر
        let _ = Self::run_ip(&["route", "del", "default"]);

        let mut args = vec!["route", "add", "default", "via", gateway];
        if let Some(dev) = device {
            args.push("dev");
            args.push(dev);
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