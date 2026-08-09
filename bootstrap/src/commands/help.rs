use shared_kernel::prelude::*;

pub struct HelpCommand;

impl Command for HelpCommand {

    fn name(&self) -> &'static str {

        "help"
    }

    fn description(&self) -> &'static str {

        "Show available commands"
    }

    fn execute(
        &self,
        _: &CommandContext,
        _: &[&str],
    ) -> String {

        [
            "Available commands:",
            " help",
            " modules",
            " system",
            " asset list",
            " asset add <name> <type>",
            " graph node <label>",
            " graph edge <from> <to> <relation>",
            " graph list",
            " graph neighbors <id>",
            " intel analyze",
            " intel list",
            " decision submit <subject-id> <critical|high|low> <description...>",
            " decision approve <id>",
            " decision reject <id>",
            " decision list",
            " firewall list",
            " firewall block <ip>",
            " firewall unblock <ip>",
            " exit",
        ].join("\n")
    }
}
