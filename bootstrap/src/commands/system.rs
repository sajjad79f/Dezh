use monitoring::MonitoringService;
use shared_kernel::prelude::*;

pub struct SystemCommand;

impl Command for SystemCommand {

    fn name(&self) -> &'static str {

        "system"
    }

    fn description(&self) -> &'static str {

        "Show system information"
    }

    fn execute(
        &self,
        _: &CommandContext,
        _: &[&str],
    ) -> String {

        let info = MonitoringService::system_info();

        format!(
            "Hostname      : {}\nOS            : {}\nKernel        : {}\nCPU Cores     : {}\nTotal Memory  : {}\nUsed Memory   : {}",
            info.hostname,
            info.os,
            info.kernel,
            info.cpu_cores,
            info.total_memory,
            info.used_memory,
        )
    }
}
