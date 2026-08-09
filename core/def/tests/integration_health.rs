use def::{
    DefFacade,
    models::HealthStatus,
};

#[test]

fn service_health() {

    let def =

        DefFacade::new();

    let health =

        def.health();

    assert!(

        matches!(

            health.status,

            HealthStatus::Healthy
        )
    );
}