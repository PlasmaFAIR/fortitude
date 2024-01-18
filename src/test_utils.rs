#[cfg(test)]
pub mod test_utils {
    use crate::core::Violation;
    use crate::parser::fortran_parser;
    use tree_sitter::Node;

    pub fn test_tree_method<S: AsRef<str>>(
        f: fn(&Node, &str) -> Vec<Violation>,
        source: S,
        expected_violations: Option<Vec<Violation>>,
    ) {
        let src = source.as_ref();
        let mut parser = fortran_parser();
        let tree = parser.parse(src, None).unwrap();
        let root = tree.root_node();
        let mut violations = f(&root, src);
        match expected_violations {
            Some(x) => {
                assert_eq!(violations.len(), x.len());
                for (actual, expected) in violations.iter().zip(&x) {
                    assert_eq!(actual, expected);
                }
            }
            None => {
                // Do nothing!
            }
        }
    }
}
