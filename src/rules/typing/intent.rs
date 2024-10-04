use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use tree_sitter::Node;

/// Rules for catching missing intent

pub struct MissingIntent {}

impl Rule for MissingIntent {
    fn new(_settings: &Settings) -> Self {
        MissingIntent {}
    }

    fn explain(&self) -> &'static str {
        "
        Procedure dummy arguments should have an explicit `intent`
        attributes. This can help catch logic errors, potentially improve
        performance, as well as serving as documentation for users of
        the procedure.

        Arguments with `intent(in)` are read-only input variables, and cannot be
        modified by the routine.

        Arguments with `intent(out)` are output variables, and their value on
        entry into the routine can be safely ignored.

        Finally, `intent(inout)` arguments can be both read and modified by the
        routine. If an `intent` is not specified, it will default to
        `intent(inout)`.
        "
    }
}

impl ASTRule for MissingIntent {
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        // Names of all the dummy arguments
        let parameters: Vec<&str> = node
            .child_by_field_name("parameters")?
            .named_children(&mut node.walk())
            .filter_map(|param| param.to_text(src))
            .collect();

        let parent = node.parent()?;
        let procedure_kind = parent.kind();

        // Logic here is:
        // 1. find variable declarations
        // 2. filter to the declarations that don't have an `intent`
        // 3. filter to the ones that contain any of the dummy arguments
        // 4. collect into a vec of violations
        //
        // We filter by missing intent first, so we only have to
        // filter by the dummy args once -- otherwise we either catch
        // local var decls on the same line, or need to iterate over
        // the decl names twice
        let violations = parent
            .named_children(&mut parent.walk())
            .filter(|child| child.kind() == "variable_declaration")
            .filter(|decl| {
                !decl
                    .children_by_field_name("attribute", &mut decl.walk())
                    .any(|attr| {
                        attr.to_text(src)
                            .unwrap_or("")
                            .to_lowercase()
                            .starts_with("intent")
                    })
            })
            .flat_map(|decl| {
                decl.children_by_field_name("declarator", &mut decl.walk())
                    .filter_map(|declarator| {
                        let identifier = match declarator.kind() {
                            "identifier" => Some(declarator),
                            "sized_declarator" => declarator.child_with_name("identifier"),
                            // Although tree-sitter-fortran grammar allows
                            // `init_declarator` and `pointer_init_declarator`
                            // here, dummy arguments aren't actually allow
                            // initialisers. _Could_ still catch them here, and
                            // flag as syntax error elsewhere?
                            _ => None,
                        }?;
                        let name = identifier.to_text(src)?;
                        if parameters.contains(&name) {
                            return Some((declarator, name));
                        }
                        None
                    })
                    .map(|(dummy, name)| {
                        let msg = format!(
                            "{procedure_kind} argument '{name}' missing 'intent' attribute"
                        );
                        Violation::from_node(msg, &dummy)
                    })
                    .collect::<Vec<Violation>>()
            })
            .collect();

        Some(violations)
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["function_statement", "subroutine_statement"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_missing_intent() -> anyhow::Result<()> {
        let source = dedent(
            "
            integer function foo(a, b, c)
              use mod
              integer :: a, c(2), f
              integer, dimension(:), intent(in) :: b
            end function

            subroutine bar(d, e, f)
              integer, pointer :: d
              integer, allocatable :: e(:, :)
              type(integer(kind=int64)), intent(inout) :: f
              integer :: g
            end subroutine
            ",
        );
        let expected: Vec<Violation> = [
            (4, 14, "function", "a"),
            (4, 17, "function", "c"),
            (9, 23, "subroutine", "d"),
            (10, 27, "subroutine", "e"),
        ]
        .iter()
        .map(|(line, col, entity, arg)| {
            let msg = format!("{entity} argument '{arg}' missing 'intent' attribute");
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = MissingIntent::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
