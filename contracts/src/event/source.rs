#[derive(Clone, Debug)]

pub struct EventSource {

    pub module: String,

    pub component: String,
}

impl EventSource {

    pub fn new(

        module: impl Into<String>,

        component: impl Into<String>,
    ) -> Self {

        Self {

            module: module.into(),

            component: component.into(),
        }
    }
}