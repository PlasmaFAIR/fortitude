use fortitude::check::check;
use fortitude::cli::{parse_args, SubCommands};
use fortitude::explain::explain;

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
