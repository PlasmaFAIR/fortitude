use tree_sitter::{Parser, Language};

#[link(name = "tree-sitter-fortran")]
extern "C" {
    fn tree_sitter_fortran() -> Language;
}

fn fortran_parser() -> Parser {
    let lang_fortran = unsafe{tree_sitter_fortran()};
    let mut parser = Parser::new();
    parser.set_language(lang_fortran).unwrap();
    parser
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fortran_parser() {
        let mut parser = fortran_parser();
        let source_code = "module test_mod\n\
                           implicit none\n\
                           character(len=*), parameter :: str = 'Hello World!'\n\
                           end module test_mod";
        let tree = parser.parse(source_code, None);
        assert!(tree.is_some())
    }
}
