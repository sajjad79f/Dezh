use dai::{AssetType, DaiService};
use shared_kernel::prelude::*;

pub struct AssetCommand;

impl Command for AssetCommand {
    fn name(&self) -> &'static str {
        "asset"
    }

    fn description(&self) -> &'static str {
        "Manage assets (DAI). Usage: asset list | asset add <name> <type>"
    }

    fn execute(
        &self,
        ctx: &CommandContext,
        args: &[&str],
    ) -> String {
        let Some(dai) = ctx.services.resolve::<DaiService>() else {
            return "DAI service not available.".to_string();
        };

        match args.first().copied() {
            Some("list") => {
                let assets = dai.list_assets();

                if assets.is_empty() {
                    "No assets registered.".to_string()
                } else {
                    assets
                        .iter()
                        .map(|a| {
                            format!(
                                "{} | {} | {:?} | {:?}",
                                a.id, a.name, a.asset_type, a.status,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }

            Some("add") => {
                if args.len() < 3 {
                    return "Usage: asset add <name> <type>".to_string();
                }

                let name = args[1];

                let asset_type = match args[2] {
                    "endpoint" => AssetType::Endpoint,
                    "server" => AssetType::Server,
                    "vm" => AssetType::VirtualMachine,
                    "firewall" => AssetType::Firewall,
                    "vpn" => AssetType::VpnGateway,
                    "user" => AssetType::User,
                    other => AssetType::Other(other.to_string()),
                };

                match dai.register_asset(name, asset_type) {
                    Ok(id) => format!("Asset registered with id {id}"),
                    Err(err) => format!("Error: {err}"),
                }
            }

            _ => "Usage: asset list | asset add <name> <type>".to_string(),
        }
    }
}