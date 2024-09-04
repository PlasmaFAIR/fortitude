#[macro_use]
mod code_style;
mod best_practices;
mod check;
mod cli;
mod code_errors;
mod core;
mod explain;
mod rules;
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
