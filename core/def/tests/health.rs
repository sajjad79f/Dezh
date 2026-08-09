use def::DefFacade;

use def::models::HealthStatus;

#[test]
fn health_ok() {

    let def =
        DefFacade::new();

    let health =
        def.health();

    match health.status {

        HealthStatus::Healthy => {}

        _ => panic!(),
    }
}