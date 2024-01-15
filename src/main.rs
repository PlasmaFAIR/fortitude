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
            match check(args) {
                Some(errors) => {
                    println!("{}", errors);
                    std::process::exit(1);
                }
                None => {
                    // No errors found, do nothing!
                }
            }
        }
        _ => {
            panic!("Not yet implemented!")
        }
    }
}
