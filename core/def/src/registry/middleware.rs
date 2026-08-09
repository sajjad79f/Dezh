use std::sync::Arc;

use contracts::Event;

use crate::error::DefResult;

pub trait Middleware: Send + Sync {

    fn handle(

        &self,

        event: &mut Event,

    ) -> DefResult<()>;
}

pub type MiddlewareRef = Arc<dyn Middleware>;