extern crate clap;
extern crate grid;
extern crate mino_core;
extern crate rand;
extern crate rustyline;
extern crate termion;
extern crate tui;
use clap::{App, SubCommand};

mod helper;
mod interactive;
mod play;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("mino_cli")
        .subcommand(SubCommand::with_name("play").alias("p"))
        .subcommand(SubCommand::with_name("interactive").alias("i"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("play") {
        return play::run();
    }

    if let Some(_matches) = matches.subcommand_matches("interactive") {
        return interactive::run();
    }

    Ok(())
}
