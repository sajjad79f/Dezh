pub mod asset;
pub mod decision;
pub mod firewall;
pub mod graph;
pub mod help;
pub mod intel;
pub mod system;

use shared_kernel::prelude::*;

use asset::AssetCommand;
use decision::DecisionCommand;
use firewall::FirewallCommand;
use graph::GraphCommand;
use help::HelpCommand;
use intel::IntelCommand;
use system::SystemCommand;

pub fn register(
    commands: &mut CommandRegistry,
) {
    commands.register(HelpCommand);
    commands.register(SystemCommand);
    commands.register(AssetCommand);
    commands.register(GraphCommand);
    commands.register(IntelCommand);
    commands.register(DecisionCommand);
    commands.register(FirewallCommand);
}