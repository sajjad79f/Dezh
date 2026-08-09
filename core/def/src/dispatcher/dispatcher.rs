use std::sync::Arc;

use contracts::{Event, EventHandler};

use crate::{
    error::*,
    models::DispatchResult,
};

pub struct Dispatcher;

impl Dispatcher {
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch(
        &self,
        event: &Event,
        handlers: Vec<Arc<dyn EventHandler>>,
    ) -> DefResult<Vec<DispatchResult>> {
        let mut results = Vec::new();

        for handler in handlers {
            match handler.handle(event) {
                Ok(()) => {
                    results.push(DispatchResult::success(handler.name()));
                }
                Err(err) => {
                    results.push(DispatchResult::failed(
                        handler.name(),
                        err.to_string(),
                    ));
                }
            }
        }

        Ok(results)
    }
}