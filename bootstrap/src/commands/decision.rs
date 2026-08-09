use dde::{AutonomyLevel, DdeService};
use shared_kernel::prelude::*;
use uuid::Uuid;
use std::sync::Arc;

pub struct DecisionCommand;

impl Command for DecisionCommand {

    fn name(&self) -> &'static str {
        "decision"
    }

    fn description(&self) -> &'static str {
        "Manage decisions (DDE). Usage: decision submit <subject-id> <critical|high|low> <description...> | decision approve <id> | decision reject <id> | decision list"
    }

    fn execute(
        &self,
        ctx: &CommandContext,
        args: &[&str],
    ) -> String {

        let Some(dde) = ctx.services.resolve::<Arc<DdeService>>() else {
            return "DDE service not available.".to_string();
        };

        match args.first().copied() {

            Some("submit") => {

                if args.len() < 4 {
                    return "Usage: decision submit <subject-id> <critical|high|low> <description...>".to_string();
                }

                let Ok(subject) = Uuid::parse_str(args[1]) else {
                    return "Invalid subject id.".to_string();
                };

                let autonomy = match args[2] {
                    "critical" => AutonomyLevel::Critical,
                    "high" => AutonomyLevel::High,
                    "low" => AutonomyLevel::Low,
                    _ => return "Autonomy must be one of: critical | high | low".to_string(),
                };

                let description = args[3..].join(" ");

                let id = dde.submit_decision(subject, description, autonomy);

                format!("Decision {id} submitted with autonomy {:?}.", autonomy)
            }

            Some("approve") => {

                let Some(id_str) = args.get(1) else {
                    return "Usage: decision approve <id>".to_string();
                };

                let Ok(id) = Uuid::parse_str(id_str) else {
                    return "Invalid decision id.".to_string();
                };

                if dde.approve(id) {
                    format!("Decision {id} approved.")
                } else {
                    "No pending decision with that id.".to_string()
                }
            }

            Some("reject") => {

                let Some(id_str) = args.get(1) else {
                    return "Usage: decision reject <id>".to_string();
                };

                let Ok(id) = Uuid::parse_str(id_str) else {
                    return "Invalid decision id.".to_string();
                };

                if dde.reject(id) {
                    format!("Decision {id} rejected.")
                } else {
                    "No pending decision with that id.".to_string()
                }
            }

            Some("list") => {

                let decisions = dde.list_decisions();

                if decisions.is_empty() {
                    "No decisions recorded yet.".to_string()
                } else {
                    decisions
                        .iter()
                        .map(|d| {
                            format!(
                                "{} | {:?} | {:?} | {} ({})",
                                d.id, d.autonomy, d.status, d.description, d.subject,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }

            _ => "Usage: decision submit <subject-id> <critical|high|low> <description...> | decision approve <id> | decision reject <id> | decision list".to_string(),
        }
    }
}
