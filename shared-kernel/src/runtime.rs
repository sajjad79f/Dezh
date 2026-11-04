//! Shared helper for running async code from sync contexts.
//!
//! تکرار این تابع در ۴ سرویس (network، firewall، routing، dai) حذف شد
//! و به‌صورت واحد اینجا نگهداری می‌شود.

/// Runs a future to completion from a synchronous context.
///
/// - اگر داخل runtime توکیو باشیم: `block_in_place` + `Handle::block_on`
/// - در غیر این صورت: یک runtime موقت current-thread می‌سازد
pub fn block_on<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
        Err(_) => {
            // اگر runtime نبود (نادر): runtime موقت
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("temp runtime");
            rt.block_on(f)
        }
    }
}