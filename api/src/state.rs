use std::sync::Arc;

use shared_kernel::prelude::*;

/// State shared between all HTTP handlers.
///
/// ModuleRegistry and CommandRegistry are only written during bootstrap
/// (see `bootstrap::bootstrap`). After that point they are read-only for
/// the rest of the process lifetime, so plain `Arc` (no Mutex) is enough
/// to share them safely between the CLI shell thread and the Axum server.
#[derive(Clone)]
pub struct AppState {
    pub modules: Arc<ModuleRegistry>,
    pub commands: Arc<CommandRegistry>,
    pub services: Arc<ServiceContainer>,
}
