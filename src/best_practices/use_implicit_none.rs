use crate::parser::fortran_language;
use crate::rules;
/// Defines rules that raise errors if implicit typing is in use.
// TODO require implicit none in interface functions (code 11)
// TODO report use of function `implicit none` when its set on the enclosing module (code 12)
use tree_sitter::{Node, Query};

const CODE10: rules::Code = rules::Code::new(rules::Category::BestPractices, 10);
const MSG10: &str = "'implicit none' should be used in all modules and programs, as implicit
    typing reduces the readability of code and increases the chances of typing errors.";

fn use_implicit_none_method(root: &Node, src: &str) -> Vec<rules::Violation> {
    let mut violations = Vec::new();
    for query_type in ["module", "submodule", "program"] {
        // Search for a module, submodule or program, and optionally an 'implicit none'.
        // Capture module, submodule or program in @mod, and 'implicit none' in @implicit-none.
        // We don't need @mod explicitly, but if @implicit-none isn't found, we'll need it
        // for error reporting.
        let query_txt = format!(
            "({} (implicit_statement (none))? @implicit-none) @mod",
            query_type
        );
        let query = Query::new(fortran_language(), query_txt.as_str()).unwrap();
        let implicit_none_index = query.capture_index_for_name("implicit-none").unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for captures in cursor
            .matches(&query, *root, src.as_bytes())
            .map(|x| x.captures)
        {
            // If the @implicit-none capture isn't found, record a violation.
            if !captures.iter().any(|&x| x.index == implicit_none_index) {
                // The only other captures must be the module, submodule, or program.
                for capture in captures {
                    violations.push(rules::Violation::from_node(
                        &capture.node,
                        CODE10,
                        format!("{} missing 'implicit none'", query_type).as_str(),
                    ));
                }
            }
        }
    }
    violations
}

pub fn use_implicit_none() -> rules::Rule {
    rules::Rule::new(
        CODE10,
        rules::Method::Tree(use_implicit_none_method),
        MSG10,
        rules::Status::Standard,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::fortran_parser;

    fn test_helper(code: &str, err: Option<Vec<String>>) {
        let mut parser = fortran_parser();
        let tree = parser.parse(&code, None).unwrap();
        let root = tree.root_node();
        let rule = use_implicit_none();
        let violations = use_implicit_none_method(&root, code);
        match err {
            Some(x) => {
                assert_eq!(violations.len(), x.len());
                for (actual, expected) in violations.iter().zip(x) {
                    assert_eq!(actual.to_string(), expected);
                }
            }
            None => {
                // Do nothing!
            }
        }
    }

    #[test]
    fn test_missing_implicit_none() {
        let code = "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ";
        let errs = [2, 6]
            .iter()
            .zip(["module", "program"])
            .map(|(line, msg)| {
                format!("Line {}: B010 {} missing 'implicit none'", line, msg,).to_string()
            })
            .collect();
        test_helper(code, Some(errs));
    }

    #[test]
    fn test_uses_implicit_none() {
        let code = "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function
            end module

            program my_program
                implicit none
                integer, paramter :: x = 2
                write(*,*) x
            end program
            ";
        test_helper(code, None);
    }
}
