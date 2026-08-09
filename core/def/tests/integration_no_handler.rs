use contracts::Event;

use def::DefFacade;

#[test]

fn no_handler_registered() {

    let def =

        DefFacade::new();

    let event = Event {

        topic:

            "unknown".into(),

        payload:

            vec![],
    };

    assert!(

        def.publish(event)

            .is_err()
    );
}