/// Critical → فقط هشدار، انسان باید عمل کند  
/// High     → پیشنهاد می‌دهد، نیاز به تأیید انسان دارد  
/// Low      → خودش اجرا می‌کند
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyLevel {
    Critical,
    High,
    Low,
}