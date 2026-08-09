use def::models::DispatchResult;

#[test]

fn dispatch_success() {

    let result =

        DispatchResult::success(

            "handler",
        );

    assert!(

        result.success,
    );
}

#[test]

fn dispatch_failure() {

    let result =

        DispatchResult::failed(

            "handler",

            "error",
        );

    assert!(

        !result.success,
    );

    assert!(

        result.error.is_some(),
    );
}