use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use contracts::{Event, EventHandler, Result};

use def::DefFacade;

struct Handler {

    counter: Arc<AtomicUsize>,
}

impl EventHandler for Handler {

    fn name(&self) -> &'static str {

        "handler"
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

fn multiple_handlers() {

    let def = DefFacade::new();

    let counter =

        Arc::new(

            AtomicUsize::new(0),
        );

    def.subscribe(

        "event",

        Arc::new(

            Handler {

                counter: counter.clone(),
            },
        ),
    );

    def.subscribe(

        "event",

        Arc::new(

            Handler {

                counter: counter.clone(),
            },
        ),
    );

    let event = Event {

        topic:

            "event".into(),

        payload:

            vec![],
    };

    let result =

        def.publish(event)

            .unwrap();

    assert_eq!(

        result.delivered,

        2,
    );

    assert_eq!(

        counter.load(

            Ordering::SeqCst,
        ),

        2,
    );
}