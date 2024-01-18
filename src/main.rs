mod best_practices;
mod check;
mod cli;
#[macro_use]
mod core;
mod parser;
mod rules;
mod settings;
mod test_utils;

use check::check;
use cli::{parse_args, SubCommands};

fn main() {
    let args = parse_args();
    match args.command {
        SubCommands::Check(args) => {
            std::process::exit(check(args));
        }
        _ => {
            panic!("Not yet implemented!")
        }
    }
}
