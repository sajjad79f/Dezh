use std::any::Any;
use std::sync::Arc;

use shared_kernel::prelude::*;

use crate::error::ApiError;

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

impl AppState {
    /// resolve کردن سرویس از container یا خطای 503 با نام گویا.
    ///
    /// به‌جای بلوک ۱۰ خطی `let Some(x) = ... else { return 503 }`،
    /// فقط: `let fw = state.service::<FirewallService>("firewall")?;`
    pub fn service<T>(&self, label: &'static str) -> Result<&T, ApiError>
    where
        T: Any + Send + Sync + 'static,
    {
        self.services
            .resolve::<T>()
            .ok_or_else(|| ApiError::service_unavailable(format!("{label} service not available")))
    }
}