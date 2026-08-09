use def::DefFacade;

#[test]
fn statistics_default() {

    let def =
        DefFacade::new();

    let statistics =
        def.statistics();

    assert_eq!(
        statistics.published_events,
        0
    );
}