extern crate clap;
extern crate cursive;
extern crate rand;
use clap::{App, SubCommand};

mod associated_trait;
mod impl_trait;
mod state_machine;
mod trait_method_return_self;
mod tui;

fn main() {
    let matches = App::new("Sandbox")
        .subcommand(SubCommand::with_name("state_machine"))
        .subcommand(SubCommand::with_name("associated_trait"))
        .subcommand(SubCommand::with_name("trait_method_return_self"))
        .subcommand(SubCommand::with_name("impl_trait"))
        .subcommand(SubCommand::with_name("tui"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("state_machine") {
        state_machine::run();
        return;
    }
    if let Some(_matches) = matches.subcommand_matches("associated_trait") {
        associated_trait::run();
        return;
    }
    if let Some(_matches) = matches.subcommand_matches("trait_method_return_self") {
        trait_method_return_self::run();
        return;
    }
    if let Some(_matches) = matches.subcommand_matches("impl_trait") {
        impl_trait::run();
        return;
    }
    if let Some(_matches) = matches.subcommand_matches("tui") {
        tui::run();
        return;
    }
}
