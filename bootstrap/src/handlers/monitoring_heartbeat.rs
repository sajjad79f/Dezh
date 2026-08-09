use std::sync::Arc;

use contracts::{Event, EventHandler};
use dai::DaiService;
use dde::{AutonomyLevel, DdeService};
use die::DieService;

/// وقتی "monitoring.heartbeat" برسه: تحلیل DIE روی Assetهای فعلی اجرا
/// می‌شه، و برای هر Finding، یه Decision با Autonomy=High (نیاز به تأیید
/// انسان) به DDE ارسال می‌شه. هنوز هیچ اقدام خودکاری انجام نمی‌ده.
pub struct MonitoringHeartbeatHandler {
    pub dai: Arc<DaiService>,
    pub die: Arc<DieService>,
    pub dde: Arc<DdeService>,
}

impl EventHandler for MonitoringHeartbeatHandler {

    fn handle(&self, _event: &Event) {

        let findings = self.die.analyze_assets(&self.dai.list_assets());

        for finding in findings {

            self.dde.submit_decision(
                finding.subject,
                finding.message,
                AutonomyLevel::High,
            );
        }
    }
}