use super::CommandContext;

pub trait Command: Send + Sync {

    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn execute(&self, context: &CommandContext, args: &[&str]) -> String;
}