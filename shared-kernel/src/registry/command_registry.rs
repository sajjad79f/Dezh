use crate::command::Command;

pub struct CommandRegistry {

    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {

    pub fn new() -> Self {

        Self {

            commands: Vec::new(),
        }
    }

    pub fn register<C>(&mut self, command: C)

    where
        C: Command + 'static,
    {
        self.commands.push(Box::new(command));
    }

    pub fn commands(&self) -> &[Box<dyn Command>] {

        &self.commands
    }
}