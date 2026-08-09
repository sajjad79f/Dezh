use def::middleware::{
    MiddlewarePipeline,
    ValidationMiddleware,
};

use def::registry::{
    MiddlewareRegistry,
    Middleware,
};

use std::sync::Arc;

#[test]

fn pipeline_can_be_created() {

    let registry = MiddlewareRegistry::new();

    registry.register(

        Arc::new(

            ValidationMiddleware,
        ),
    );

    let _ =

        MiddlewarePipeline::new(

            &registry,
        );
}