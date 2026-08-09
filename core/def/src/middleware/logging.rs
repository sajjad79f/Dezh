use tracing::debug;

use contracts::Event;

use crate::{
    error::DefResult,
    registry::Middleware,
};

pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn handle(&self, event: &mut Event) -> DefResult<()> {
        debug!(
            topic = %event.topic,
            "event received",
        );
        Ok(())
    }
}