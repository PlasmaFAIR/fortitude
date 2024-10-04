use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use itertools::Itertools;
use tree_sitter::Node;

/// Rules for catching assumed size variables

pub struct AssumedSize {}

impl Rule for AssumedSize {
    fn new(_settings: &Settings) -> Self {
        AssumedSize {}
    }

    fn explain(&self) -> &'static str {
        "
        Assumed size dummy arguments declared with a star `*` as the size should be
        avoided, and assumed shape with a colon `:` should be used instead. There are
        several downsides to assumed size, the main one being that the compiler is
        not able to determine the array bounds, so it is not possible to check for
        array overruns or to use the array in whole-array expressions.

        Instead, prefer assumed shape arguments, as the compiler is able to keep track of
        the upper bounds automatically, and pass this information under the hood. It also
        allows use of whole-array expressions, such as `a = b + c`, where `a, b, c` are
        all arrays of the same shape.

        Instead of:

        ```
        subroutine process_array(array)
            integer, dimension(*), intent(in) :: array
            ...
        ```

        use:

        ```
        subroutine process_array(array)
            integer, dimension(:), intent(in) :: array
            ...
        ```
        "
    }
}

impl ASTRule for AssumedSize {
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Assumed size ok for parameters
        if declaration
            .children_by_field_name("attribute", &mut declaration.walk())
            .filter_map(|attr| attr.to_text(src))
            .any(|attr_name| attr_name.to_lowercase() == "parameter")
        {
            return None;
        }

        // Are we looking at something declared like `array(*)`?
        if let Some(sized_decl) = node
            .ancestors()
            .find(|parent| parent.kind() == "sized_declarator")
        {
            let identifier = sized_decl.child_with_name("identifier")?;
            let name = identifier.to_text(src)?;
            let msg = format!("'{name}' has assumed size");
            return some_vec![Violation::from_node(msg, node)];
        }

        // Collect things that look like `dimension(*)` -- this
        // applies to all identifiers on this line
        let all_decls = declaration
            .children_by_field_name("declarator", &mut declaration.walk())
            .filter_map(|declarator| {
                let identifier = match declarator.kind() {
                    "identifier" => Some(declarator),
                    "sized_declarator" => declarator.child_with_name("identifier"),
                    _ => None,
                }?;
                identifier.to_text(src)
            })
            .map(|name| Violation::from_node(format!("'{name}' has assumed size"), node))
            .collect_vec();

        Some(all_decls)
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["assumed_size"]
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
    fn test_assumed_size() -> anyhow::Result<()> {
        let source = dedent(
            "
            subroutine assumed_size_dimension(array, n, m, l, o, p, options, thing)
              integer, intent(in) :: n, m
              integer, dimension(n, m, *), intent(in) :: array
              integer, intent(in) :: l(*), o, p(*)
              character(len=*) :: options
              character(*) :: thing
              ! following are ok because they're parameters
              integer, dimension(*), parameter :: param = [1, 2, 3]
              character(*), dimension(*), parameter :: param_char = ['hello']
            end subroutine assumed_size_dimension
            ",
        );
        let expected: Vec<Violation> = [
            (4, 28, "array"),
            (5, 28, "l"),
            (5, 37, "p"),
            (6, 17, "options"),
            (7, 13, "thing"),
        ]
        .iter()
        .map(|(line, col, variable)| {
            let msg = format!("'{variable}' has assumed size");
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = AssumedSize::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
