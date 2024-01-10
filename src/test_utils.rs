#[cfg(test)]
mod test_utils {
    use parser::fortran_parser;
    use rules::{Category, Code, Violation};

    pub const TEST_CODE: Code = Code::new(Category::BestPractices, 999);

    pub fn test_tree_method(
        f: fn(&Node, &str) -> Vec<Violation>,
        source: &str,
        expected_violations: Option<Violation>>,
    ) {
        let mut parser = fortran_parser();
        let tree = parser.parse(&code, None).unwrap();
        let root = tree.root_node();
        let rule = use_implicit_none();
        let mut violations = f(TEST_CODE, &root, code);
        violations.sort();
        match expected_violations {
            Some(x) => {
                assert_eq!(violations.len(), x.len());
                for (actual, expected) in violations.iter().zip(x) {
                    assert_eq!(actual, expected);
                }
            }
            None => {
                // Do nothing!
            }
        }
    }
}
