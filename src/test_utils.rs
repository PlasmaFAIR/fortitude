#[cfg(test)]
pub mod test_utils {
    use crate::parser::fortran_parser;
    use crate::rules::{Category, Code, Violation};
    use tree_sitter::Node;

    pub const TEST_CODE: Code = Code::new(Category::BestPractices, 255);

    pub fn test_tree_method(
        f: fn(Code, &Node, &str) -> Vec<Violation>,
        source: &str,
        expected_violations: Option<Vec<Violation>>,
    ) {
        let mut parser = fortran_parser();
        let tree = parser.parse(&source, None).unwrap();
        let root = tree.root_node();
        let mut violations = f(TEST_CODE, &root, source);
        violations.sort();
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
