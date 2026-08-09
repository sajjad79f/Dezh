use contracts::Event;

use crate::{
    error::DefResult,
    registry::Middleware,
};

pub struct CorrelationMiddleware;

impl Middleware for CorrelationMiddleware {

    fn handle(

        &self,

        _event: &mut Event,

    ) -> DefResult<()> {

        //
        // بعداً CorrelationId
        // TraceId
        // ParentId
        // اینجا مدیریت می‌شوند.
        //

        Ok(())
    }
}