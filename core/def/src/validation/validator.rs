use contracts::Event;

use crate::error::*;

pub struct EventValidator;

impl EventValidator {

    pub fn validate(

        event: &Event,

    ) -> DefResult<()> {

        if event.topic.as_str().trim().is_empty() {

            return Err(

                DefError::Validation(

                    String::from("topic cannot be empty"),

                ),

            );
        }

        Ok(())
    }
}