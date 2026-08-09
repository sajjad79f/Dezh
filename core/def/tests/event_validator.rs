use contracts::Event;

use def::validation::EventValidator;

#[test]

fn validate_event() {

    let event = Event {

        topic:

            "system.start".into(),

        payload:

            vec![],
    };

    assert!(

        EventValidator::validate(

            &event,
        )

        .is_ok()
    );
}

#[test]

fn empty_topic() {

    let event = Event {

        topic:

            "".into(),

        payload:

            vec![],
    };

    assert!(

        EventValidator::validate(

            &event,
        )

        .is_err()
    );
}