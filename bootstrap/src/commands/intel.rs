use dai::DaiService;
use die::DieService;
use shared_kernel::prelude::*;

pub struct IntelCommand;

impl Command for IntelCommand {
    fn name(&self) -> &'static str {
        "intel"
    }

    fn description(&self) -> &'static str {
        "Run analysis (DIE) over current assets. Usage: intel analyze | intel list"
    }

    fn execute(
        &self,
        ctx: &CommandContext,
        args: &[&str],
    ) -> String {
        let Some(die) = ctx.services.resolve::<DieService>() else {
            return "DIE service not available.".to_string();
        };

        match args.first().copied() {
            Some("analyze") => {
                let Some(dai) = ctx.services.resolve::<DaiService>() else {
                    return "DAI service not available.".to_string();
                };

                let findings = die.analyze_assets(&dai.list_assets());

                if findings.is_empty() {
                    "Analysis complete. No new findings.".to_string()
                } else {
                    let mut lines =
                        vec![format!("Analysis complete. {} finding(s):", findings.len())];

                    lines.extend(findings.iter().map(|f| {
                        format!("  [{:?}] {} ({})", f.severity, f.message, f.subject)
                    }));

                    lines.join("\n")
                }
            }

            Some("list") => {
                let findings = die.list_findings();

                if findings.is_empty() {
                    "No findings recorded yet.".to_string()
                } else {
                    findings
                        .iter()
                        .map(|f| {
                            format!(
                                "{} | [{:?}] {} ({})",
                                f.id, f.severity, f.message, f.subject
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }

            _ => "Usage: intel analyze | intel list".to_string(),
        }
    }
}