use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct ServiceContainer {

    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceContainer {

    pub fn new() -> Self {

        Self {

            services: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, service: T)

    where
        T: Any + Send + Sync + 'static,
    {
        self.services
            .insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn resolve<T>(&self) -> Option<&T>

    where
        T: Any + Send + Sync + 'static,
    {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<T>())
    }

    pub fn contains<T>(&self) -> bool

    where
        T: Any + Send + Sync + 'static,
    {
        self.services.contains_key(&TypeId::of::<T>())
    }
}