use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EventId(Uuid);

impl EventId {

    pub fn new() -> Self {

        Self(Uuid::new_v4())
    }
}

impl fmt::Display for EventId {

    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {

        write!(f, "{}", self.0)
    }
}