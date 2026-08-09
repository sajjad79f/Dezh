use def::prelude::*;

#[test]
fn statistics_are_created() {

    let statistics = Statistics::default();

    assert_eq!(

        statistics.published_events,

        0
    );
}

#[test]
fn health_is_healthy() {

    let health = Health::healthy();

    match health.status {

        HealthStatus::Healthy => {}

        _ => panic!("invalid state"),
    }
}

#[test]
fn publish_result_success() {

    let result = PublishResult::success(

        10,

        std::time::Duration::from_millis(1),
    );

    assert!(result.is_success());
}