mod active_rules;
mod best_practices;
mod check;
mod cli;
mod parser;
mod rules;
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
