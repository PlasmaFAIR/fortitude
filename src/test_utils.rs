#[cfg(test)]
pub mod test_utils {
    use crate::{Method, Rule, Violation};
    use tree_sitter::Node;

    fn scan_tree<F>(f: &F, entrypoints: &Vec<&str>, node: &Node, source: &str) -> Vec<Violation>
    where
        F: Fn(&Node, &str) -> Option<Violation>,
    {
        let mut violations = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if entrypoints.contains(&child.kind()) {
                if let Some(violation) = f(&child, source) {
                    violations.push(violation);
                }
            }
            violations.extend(scan_tree(f, entrypoints, &child, source));
        }
        violations
    }

    pub fn test_tree_method<S: AsRef<str>>(
        rule: &dyn Rule,
        source: S,
        expected_violations: Option<Vec<Violation>>,
    ) -> Result<(), String>
    where
        S: AsRef<str>,
    {
        let src = source.as_ref();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_fortran::language())
            .expect("Error loading Fortran grammar");
        let tree = parser.parse(src, None).unwrap();
        let root = tree.root_node();
        let mut violations = Vec::new();
        let entrypoints = rule.entrypoints();

        match rule.method() {
            Method::Tree(f) => {
                violations.extend(scan_tree(&f, &entrypoints, &root, src));
            }
            _ => {
                return Err("Broken entrypoints".to_string());
            }
        }

        if let Some(x) = expected_violations {
            assert_eq!(violations.len(), x.len());
            for (actual, expected) in violations.iter().zip(&x) {
                assert_eq!(actual, expected);
            }
        }
        Ok(())
    }
}
