use def::telemetry::TelemetryMetrics;

#[test]

fn telemetry_publish() {

    let metrics =

        TelemetryMetrics::new();

    metrics.record_publish();

    metrics.record_dispatch();

    metrics.record_failure();

    let statistics =

        metrics.statistics(

            0,

            0,

            0,
        );

    assert_eq!(

        statistics.published_events,

        1,
    );

    assert_eq!(

        statistics.dispatched_events,

        1,
    );

    assert_eq!(

        statistics.failed_events,

        1,
    );
}