use def::router::TopicMatcher;

#[test]
fn exact_match() {

    assert!(

        TopicMatcher::matches(

            "system.start",

            "system.start",
        )
    );
}

#[test]
fn wildcard_match() {

    assert!(

        TopicMatcher::matches(

            "*",

            "anything",
        )
    );
}

#[test]
fn mismatch() {

    assert!(

        !TopicMatcher::matches(

            "firewall",

            "vpn",
        )
    );
}