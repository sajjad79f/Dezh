#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatus {
    /// Critical: فقط هشدار داده شده
    Alerted,
    /// High: منتظر تأیید انسان
    PendingApproval,
    Approved,
    Rejected,
    /// Low: توسط خود DDE اجرا شده
    AutoExecuted,
}