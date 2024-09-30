use crate::ast::to_text;
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
        Dummy arguments should have an explicit `intent` attribute. This can help
        catch logic errors, and potentially improve performance
        "
    }
}

impl ASTRule for MissingIntent {
    fn check<'a>(&self, node: &Node, src: &'a str) -> Option<Vec<Violation>> {
        // Names of all the dummy arguments
        let parameters: Vec<&str> = node
            .child_by_field_name("parameters")?
            .named_children(&mut node.walk())
            .filter_map(|param| Some(to_text(&param, &src)?))
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
                        to_text(&attr, &src)
                            .unwrap_or("")
                            .to_lowercase()
                            .starts_with("intent")
                    })
            })
            .filter_map(|decl| {
                let dummys: Vec<(Node, &str)> = decl
                    .children_by_field_name("declarator", &mut decl.walk())
                    .filter_map(|declarator| {
                        let name = to_text(&declarator, &src)?;
                        // FIXME: need to extract identifier from
                        // sized_declarators etc
                        if parameters.contains(&name) {
                            return Some((declarator, name));
                        }
                        None
                    })
                    .collect();

                let violations: Vec<Violation> = dummys
                    .iter()
                    .filter_map(|(dummy, name)| {
                        let msg = format!(
                            "{procedure_kind} argument '{name}' missing 'intent' attribute"
                        );
                        Some(Violation::from_node(&msg, &dummy))
                    })
                    .collect();
                Some(violations)
            })
            .flatten()
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
            integer function foo(a, b, c, d)
              use mod
              integer :: a, c, e
              integer, dimension(:), intent(in) :: b
              integer, pointer, dimension(:) :: d
              integer :: f
            end function
            ",
        );
        let expected: Vec<Violation> = [
            (4, 14, "function", "a"),
            (4, 17, "function", "c"),
            (6, 37, "function", "d"),
        ]
        .iter()
        .map(|(line, col, entity, arg)| {
            let msg = format!("{entity} argument '{arg}' missing 'intent' attribute");
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = MissingIntent::new(&default_settings());
        let actual = rule.apply(&source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
