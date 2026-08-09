use std::sync::Arc;

use firewall::FirewallService;
use shared_kernel::prelude::*;

pub struct FirewallCommand;

impl Command for FirewallCommand {

    fn name(&self) -> &'static str {
        "firewall"
    }

    fn description(&self) -> &'static str {
        "Manage the OS firewall via nftables. Usage: firewall list | firewall block <ip> | firewall unblock <ip>"
    }

    fn execute(
        &self,
        ctx: &CommandContext,
        args: &[&str],
    ) -> String {

        let Some(firewall) = ctx.services.resolve::<Arc<FirewallService>>() else {
            return "Firewall service not available.".to_string();
        };

        match args.first().copied() {

            Some("list") => match firewall.list_rules() {
                Ok(output) => output,
                Err(err) => format!("Error: {err}"),
            },

            Some("block") => {

                let Some(ip) = args.get(1) else {
                    return "Usage: firewall block <ip>".to_string();
                };

                match firewall.block_ip(ip) {
                    Ok(()) => format!("Blocked {ip}."),
                    Err(err) => format!("Error: {err}"),
                }
            }

            Some("unblock") => {

                let Some(ip) = args.get(1) else {
                    return "Usage: firewall unblock <ip>".to_string();
                };

                match firewall.unblock_ip(ip) {
                    Ok(()) => format!("Unblocked {ip}."),
                    Err(err) => format!("Error: {err}"),
                }
            }

            _ => "Usage: firewall list | firewall block <ip> | firewall unblock <ip>".to_string(),
        }
    }
}