use tree_sitter::{Language, Parser};

#[link(name = "tree-sitter-fortran")]
extern "C" {
    fn tree_sitter_fortran() -> Language;
}

pub fn fortran_language() -> Language {
    unsafe { tree_sitter_fortran() }
}

pub fn fortran_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(fortran_language())
        .expect("Failed to set up Fortan parser");
    parser
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fortran_parser() {
        let mut parser = fortran_parser();
        let source_code = "
            module test_mod
               implicit none
               character(len=*), parameter :: str = 'Hello World!'
           end module test_mod
           ";
        let tree = parser.parse(source_code, None);
        assert!(tree.is_some())
    }
}
