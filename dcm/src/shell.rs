use std::io::{self, Write};
use std::sync::Arc;

use shared_kernel::prelude::*;

pub fn run(

    modules: &ModuleRegistry,

    commands: &CommandRegistry,

    services: &Arc<ServiceContainer>,
) {

    let ctx = CommandContext::new(services.clone());

    loop {

        print!("DEZH> ");

        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input.is_empty() {

            continue;
        }

        if input == "exit" {

            println!("Bye.");

            break;
        }

        if input == "modules" {

            println!();

            for descriptor in modules.descriptors() {

                println!(
                    "{} {}",
                    descriptor.name,
                    descriptor.version
                );
            }

            println!();

            continue;
        }

        let mut handled = false;

        let args: Vec<&str> = input.split_whitespace().collect();

        for command in commands.commands() {

            if command.name() == args[0] {

                let output = command.execute(
                    &ctx,
                    &args[1..],
                );

                println!();
                println!("{}", output);
                println!();

                handled = true;

                break;
            }
        }

        if !handled {

            println!("Unknown command.");
        }
    }
}
