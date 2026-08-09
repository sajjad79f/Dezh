use std::sync::RwLock;

use super::MiddlewareRef;

#[derive(Default)]
pub struct MiddlewareRegistry {
    middleware: RwLock<Vec<MiddlewareRef>>,
}

impl MiddlewareRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, middleware: MiddlewareRef) {
        self.middleware
            .write()
            .unwrap()
            .push(middleware);
    }

    pub fn all(&self) -> Vec<MiddlewareRef> {
        self.middleware.read().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.middleware.read().unwrap().len()
    }
}