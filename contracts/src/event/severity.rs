#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]

pub enum Severity {

    Trace,

    Debug,

    Info,

    Warning,

    Error,

    Critical,
}