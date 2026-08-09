use std::sync::Arc;

use contracts::Event;

use def::registry::{
    Middleware,
    MiddlewareRegistry,
};

use def::error::DefResult;

struct TestMiddleware;

impl Middleware for TestMiddleware {

    fn handle(

        &self,

        _: &mut Event,

    ) -> DefResult<()> {

        Ok(())
    }
}

#[test]
fn middleware_register() {

    let registry =

        MiddlewareRegistry::new();

    registry.register(

        Arc::new(TestMiddleware),
    );

    assert_eq!(

        registry.count(),

        1,
    );
}