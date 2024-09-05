#[macro_use]
mod rules;
mod rule_set;
mod check;
mod cli;
mod core;
mod explain;
mod settings;
mod test_utils;

use check::check;
use cli::{parse_args, SubCommands};
use explain::explain;

fn main() {
    let args = parse_args();
    match args.command {
        SubCommands::Check(args) => {
            std::process::exit(check(args));
        }
        SubCommands::Explain(args) => {
            std::process::exit(explain(args));
        }
    }
}
