mod parser;
mod rules;
mod best_practices;

use std::env;
use std::fs;
use parser::fortran_parser;
use best_practices::use_modules::use_modules_method;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let content = fs::read_to_string(filename).unwrap();
    //println!("{}", &content);
    
    let mut parser = fortran_parser();
    let tree = parser.parse(&content, None).unwrap();
    let root = tree.root_node();
    println!("{}", tree.root_node().to_sexp());
    
    // Check that functions are scoped properly
    let violations = use_modules_method(&root);
    for violation in violations.iter() {
        println!("{}", violation);
    }
}
