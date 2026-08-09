use super::{Health, Statistics};

#[derive(Debug, Clone)]

pub struct Snapshot {

    pub statistics: Statistics,

    pub health: Health,
}

impl Snapshot {

    pub fn new(

        statistics: Statistics,

        health: Health,

    ) -> Self {

        Self {

            statistics,

            health,
        }
    }
}