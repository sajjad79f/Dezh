pub mod error;
pub mod models;
pub mod registry;
pub mod service;
pub mod facade;
pub mod prelude;

pub use error::{DdeError, DdeResult};
pub use facade::DdeFacade;
pub use models::{AutonomyLevel, Decision, DecisionStatus};
pub use service::DdeService;