use contracts::Event;

use def::validation::EventValidator;

#[test]
fn validate_event() {
    let event = Event::simple("system.start", "");

    assert!(EventValidator::validate(&event).is_ok());
}

#[test]
fn empty_topic() {
    let event = Event::simple("", "");

    assert!(EventValidator::validate(&event).is_err());
}
