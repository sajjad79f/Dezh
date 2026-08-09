use std::sync::Arc;

use crate::service_container::ServiceContainer;

pub struct CommandContext {
    pub services: Arc<ServiceContainer>,
}

impl CommandContext {
    pub fn new(services: Arc<ServiceContainer>) -> Self {
        Self { services }
    }
}
