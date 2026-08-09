use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use contracts::{Event, EventHandler, Result};

use def::DefFacade;

struct TestHandler {

    counter: Arc<AtomicUsize>,
}

impl EventHandler for TestHandler {

    fn name(&self) -> &'static str {

        "integration-handler"
    }

    fn handle(

        &self,

        _: &Event,

    ) -> Result<()> {

        self.counter.fetch_add(

            1,

            Ordering::SeqCst,
        );

        Ok(())
    }
}

#[test]

fn publish_event() {

    let def = DefFacade::new();

    let counter =

        Arc::new(

            AtomicUsize::new(0),
        );

    def.subscribe(

        "system.start",

        Arc::new(

            TestHandler {

                counter: counter.clone(),
            },
        ),
    );

    let event = Event {

        topic:

            "system.start".into(),

        payload:

            vec![],
    };

    let result =

        def.publish(event)

            .unwrap();

    assert_eq!(

        result.delivered,

        1,
    );

    assert_eq!(

        counter.load(

            Ordering::SeqCst,
        ),

        1,
    );
}