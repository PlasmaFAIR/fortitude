use tree_sitter::{Parser, Language};

extern "C" {fn tree_sitter_fortran() -> Language;}

fn main() {
    println!("Hello, world!");
}

#[test]
fn test_parser() {
    let lang_fortran = unsafe{tree_sitter_fortran()};
    let mut parser = Parser::new();
    parser.set_language(lang_fortran).unwrap();

    let source_code = "module test_mod\n\
                       implicit none\n\
                       character(len=*), parameter :: str = 'Hello World!'\n\
                       end module test_mod";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", tree.root_node().to_sexp());
}
