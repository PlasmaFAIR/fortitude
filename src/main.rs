mod best_practices;
mod parser;
mod rules;

use best_practices::add_best_practices_rules;
use parser::fortran_parser;
use std::collections::HashSet;
use std::env;
use std::fs;

fn main() {
    // Currently expects one file provided via the command line
    // TODO Write proper command line interface
    // TODO Enable running on multiple files
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let content = fs::read_to_string(filename).unwrap();

    // Parse the file, extract the root node
    let mut parser = fortran_parser();
    let tree = parser.parse(&content, None).unwrap();
    let root = tree.root_node();
    //println!("{}", root.to_sexp());

    // Collect available rules
    // TODO Add feature to deselect rules, or add non-default ones.
    //      Later requires RuleStatus enum (default, deprecated, etc.) and RulesRegistry
    // TODO Separate rules into multiple categories:
    //      - Call syntax error rules first, and if any are found don't bother checking any others.
    //      - Match rules based on method type, and feed different views of the code to each. Tree
    //        rules should be given the root node, Line rules should be applied sequentially
    //        to each line of the file, and File rules should be given the whole file as text.
    let mut rules = HashSet::new();
    add_best_practices_rules(&mut rules);

    // Gather violations
    let mut violations = Vec::new();
    for rule in rules {
        match rule.method() {
            rules::Method::Tree(f) => {
                violations.extend(f(&root));
            }
            _ => {
                panic!(); // TODO Add extra rule types
            }
        }
    }

    // If any violations found, sort and print. Otherwise, print something nice!
    if !violations.is_empty() {
        for violation in violations.iter() {
            println!("{}", violation);
        }
        std::process::exit(1);
    }
    println!("No issues found!");
}
