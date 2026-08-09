use std::time::Duration;

use def::models::PublishResult;

#[test]

fn success_result() {

    let result =

        PublishResult::success(

            5,

            Duration::ZERO,
        );

    assert!(

        result.is_success(),
    );
}

#[test]

fn failed_result() {

    let result =

        PublishResult::failed(

            2,

            3,

            Duration::ZERO,
        );

    assert_eq!(

        result.failed,

        3,
    );
}