use firewall::{FirewallService, Protocol, RuleAction, RuleDirection};
use shared_kernel::prelude::*;
use uuid::Uuid;

pub struct FirewallCommand;

impl Command for FirewallCommand {
    fn name(&self) -> &'static str {
        "firewall"
    }

    fn description(&self) -> &'static str {
        "Manage firewall rules. Usage: firewall list | firewall add <name> <allow|deny|drop> <in|out|both> <tcp|udp|any> <src> <dst> [port] | firewall remove <id> | firewall block <ip> | firewall unblock <ip> | firewall enable <id> | firewall disable <id>"
    }

    fn execute(&self, ctx: &CommandContext, args: &[&str]) -> String {
        let Some(fw) = ctx.services.resolve::<FirewallService>() else {
            return "Firewall service not available.".to_string();
        };

        match args.first().copied() {
            Some("list") => {
                let rules = fw.list_rules();
                if rules.is_empty() {
                    return "No firewall rules.".to_string();
                }
                rules
                    .iter()
                    .map(|r| {
                        let port = r
                            .port
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "*".into());
                        let status = if r.enabled { "ON" } else { "OFF" };
                        format!(
                            "{} | [{status}] pri={} {:?} {:?} {:?} {}:{} -> {} | {}",
                            r.id,
                            r.priority,
                            r.action,
                            r.direction,
                            r.protocol,
                            r.source,
                            port,
                            r.destination,
                            r.name,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            Some("add") => {
                // firewall add <name> <action> <dir> <proto> <src> <dst> [port]
                if args.len() < 7 {
                    return "Usage: firewall add <name> <allow|deny|drop|reject> <in|out|both> <tcp|udp|icmp|any> <src> <dst> [port]".to_string();
                }

                let name = args[1];
                let action = match args[2] {
                    "allow" => RuleAction::Allow,
                    "deny" => RuleAction::Deny,
                    "drop" => RuleAction::Drop,
                    "reject" => RuleAction::Reject,
                    _ => return "Action must be: allow | deny | drop | reject".to_string(),
                };
                let direction = match args[3] {
                    "in" => RuleDirection::Inbound,
                    "out" => RuleDirection::Outbound,
                    "both" => RuleDirection::Both,
                    _ => return "Direction must be: in | out | both".to_string(),
                };
                let protocol = match args[4] {
                    "tcp" => Protocol::Tcp,
                    "udp" => Protocol::Udp,
                    "icmp" => Protocol::Icmp,
                    "any" => Protocol::Any,
                    _ => return "Protocol must be: tcp | udp | icmp | any".to_string(),
                };
                let source = args[5];
                let destination = args[6];
                let port = args.get(7).and_then(|p| p.parse::<u16>().ok());

                match fw.add_rule(name, action, direction, protocol, source, destination, port, 100) {
                    Ok(id) => format!("Rule created: {id}"),
                    Err(e) => format!("Error: {e}"),
                }
            }

            Some("remove") => {
                let Some(id_str) = args.get(1) else {
                    return "Usage: firewall remove <id>".to_string();
                };
                let Ok(id) = Uuid::parse_str(id_str) else {
                    return "Invalid rule id.".to_string();
                };
                match fw.remove_rule(id) {
                    Ok(()) => format!("Rule {id} removed."),
                    Err(e) => format!("Error: {e}"),
                }
            }

            Some("block") => {
                let Some(ip) = args.get(1) else {
                    return "Usage: firewall block <ip>".to_string();
                };
                match fw.block_ip(ip) {
                    Ok(id) => format!("Blocked {ip} (rule {id})."),
                    Err(e) => format!("Error: {e}"),
                }
            }

            Some("unblock") => {
                let Some(ip) = args.get(1) else {
                    return "Usage: firewall unblock <ip>".to_string();
                };
                match fw.unblock_ip(ip) {
                    Ok(n) => format!("Removed {n} rule(s) for {ip}."),
                    Err(e) => format!("Error: {e}"),
                }
            }

            Some("enable") => {
                let Some(id_str) = args.get(1) else {
                    return "Usage: firewall enable <id>".to_string();
                };
                let Ok(id) = Uuid::parse_str(id_str) else {
                    return "Invalid rule id.".to_string();
                };
                match fw.enable_rule(id) {
                    Ok(()) => format!("Rule {id} enabled."),
                    Err(e) => format!("Error: {e}"),
                }
            }

            Some("disable") => {
                let Some(id_str) = args.get(1) else {
                    return "Usage: firewall disable <id>".to_string();
                };
                let Ok(id) = Uuid::parse_str(id_str) else {
                    return "Invalid rule id.".to_string();
                };
                match fw.disable_rule(id) {
                    Ok(()) => format!("Rule {id} disabled."),
                    Err(e) => format!("Error: {e}"),
                }
            }

            _ => {
                "Usage: firewall list | add | remove | block | unblock | enable | disable"
                    .to_string()
            }
        }
    }
}