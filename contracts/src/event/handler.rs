use crate::{Event, DezhResult};

pub trait EventHandler: Send + Sync {
    /// نام یکتای handler برای telemetry و لاگ
    fn name(&self) -> &'static str {
        "anonymous"
    }

    fn handle(&self, event: &Event) -> DezhResult<()>;
}