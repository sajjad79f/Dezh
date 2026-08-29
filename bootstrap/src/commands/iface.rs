use firewall::{FirewallService, Zone};
use shared_kernel::prelude::*;

pub struct IfaceCommand;

impl Command for IfaceCommand {
    fn name(&self) -> &'static str {
        "iface"
    }

    fn description(&self) -> &'static str {
        "Manage network interfaces and zones. Usage: iface list | iface zone <name> <lan|wan|dmz>"
    }

    fn execute(&self, ctx: &CommandContext, args: &[&str]) -> String {
        let Some(fw) = ctx.services.resolve::<FirewallService>() else {
            return "Firewall service not available.".to_string();
        };

        match args.first().copied() {
            Some("list") => match fw.list_interfaces() {
                Ok(interfaces) if interfaces.is_empty() => "No interfaces found.".to_string(),
                Ok(interfaces) => interfaces
                    .iter()
                    .map(|i| {
                        let state = if i.up { "UP" } else { "DOWN" };
                        let addrs = if i.addresses.is_empty() {
                            "-".to_string()
                        } else {
                            i.addresses.join(", ")
                        };
                        format!("{} | {} | [{state}] | {addrs}", i.name, i.zone)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => format!("Error: {e}"),
            },

            Some("zone") => {
                if args.len() < 3 {
                    return "Usage: iface zone <name> <lan|wan|dmz>".to_string();
                }

                let Some(zone) = Zone::parse(args[2]) else {
                    return "Zone must be: lan | wan | dmz".to_string();
                };

                match fw.set_zone(args[1], zone) {
                    Ok(()) => format!("Interface '{}' assigned to zone '{}'.", args[1], zone),
                    Err(e) => format!("Error: {e}"),
                }
            }

            _ => "Usage: iface list | iface zone <name> <lan|wan|dmz>".to_string(),
        }
    }
}