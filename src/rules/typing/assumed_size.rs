use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use itertools::Itertools;
use ruff_source_file::SourceFile;
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
        avoided. There are several downsides to assumed size, the main one being
        that the compiler is not able to determine the array bounds, so it is not
        possible to check for array overruns or to use the array in whole-array
        expressions.

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

        Note that this doesn't apply to `character` types, where `character(len=*)` is
        actually the most appropriate specification for `intent(in)` arguments! This is
        because `character(len=:)` must be either a `pointer` or `allocatable`.
        "
    }
}

impl ASTRule for AssumedSize {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let src = src.source_text();
        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Deal with `character([len=]*)` elsewhere
        if let Some(dtype) = declaration.parse_intrinsic_type() {
            let is_character = dtype.to_lowercase() == "character";
            let is_kind = node.ancestors().any(|parent| parent.kind() == "kind");
            if is_character && is_kind {
                return None;
            }
        }

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

pub struct AssumedSizeCharacterIntent {}

impl Rule for AssumedSizeCharacterIntent {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        Character dummy arguments with an assumed size should only have `intent(in)`, as
        this can cause data loss with `intent([in]out)`. For example:

        ```
        program example
          character(len=3) :: short_text
          call set_text(short_text)
          print*, short_text
        contains
          subroutine set_text(text)
            character(*), intent(out) :: text
            text = \"longer than 3 characters\"
          end subroutine set_text
        end program
        ```

        Here, `short_text` will only contain the truncated \"lon\".

        To handle dynamically setting `character` sizes, use `allocatable` instead:

        ```
        program example
          character(len=3) :: short_text
          call set_text(short_text)
          print*, short_text
        contains
          subroutine set_text(text)
            character(len=:), allocatable, intent(out) :: text
            text = \"longer than 3 characters\"
          end subroutine set_text
        end program
        ```
        "
    }
}

impl ASTRule for AssumedSizeCharacterIntent {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let src = src.source_text();
        // TODO: This warning will also catch:
        // - non-dummy arguments -- these are always invalid, should be a separate warning?

        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Only applies to `character`
        if declaration.parse_intrinsic_type()?.to_lowercase() != "character" {
            return None;
        }

        // Handle `character*(*)` elsewhere -- note this just skips emitting a warning
        // for the first `*`, we'll still get one for the second `*`, but this is desired
        if let Some(sibling) = node.next_named_sibling() {
            if sibling.kind() == "assumed_size" {
                return None;
            }
        }

        let attrs_as_text = declaration
            .children_by_field_name("attribute", &mut declaration.walk())
            .filter_map(|attr| attr.to_text(src))
            .map(|attr| attr.to_lowercase())
            .collect_vec();

        // Assumed size ok for parameters
        if attrs_as_text.iter().any(|attr| attr == "parameter") {
            return None;
        }

        // Ok for `intent(in)` only
        if let Some(intent) = attrs_as_text.iter().find(|attr| attr.starts_with("intent")) {
            let intent = intent.split_whitespace().collect_vec().join("");
            if intent == "intent(in)" {
                return None;
            }
        }

        // Collect all declarations on this line
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
            .map(|name| {
                let msg =
                    format!("character '{name}' has assumed size but does not have `intent(in)`");
                Violation::from_node(msg, node)
            })
            .collect_vec();

        Some(all_decls)
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["assumed_size"]
    }
}

pub struct DeprecatedAssumedSizeCharacter {}

impl Rule for DeprecatedAssumedSizeCharacter {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        The syntax `character*(*)` is a deprecated form of `character(len=*)`. Prefer the
        second form.
        "
    }
}

impl ASTRule for DeprecatedAssumedSizeCharacter {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let src = src.source_text();
        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Only applies to `character`
        if declaration.parse_intrinsic_type()?.to_lowercase() != "character" {
            return None;
        }

        // Are we immediately (modulo whitespace) in front of `(...)`?
        if node.next_sibling()?.kind() != "(" {
            return None;
        }

        // Collect all declarations on this line
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
            .map(|name| {
                let msg = format!("character '{name}' uses deprecated syntax for assumed size");
                Violation::from_node(msg, node)
            })
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
    use crate::{settings::default_settings, test_file};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_assumed_size() -> anyhow::Result<()> {
        let source = test_file(
            "
            subroutine assumed_size_dimension(array, n, m, l, o, p, options, thing, q)
              integer, intent(in) :: n, m
              integer, dimension(n, m, *), intent(in) :: array
              integer, intent(in) :: l(*), o, p(*)
              ! warning must be on the array part for characters
              character(len=*), dimension(*) :: options
              character(*) :: thing(*)
              ! this is ok
              character(*), intent(in) :: q
              ! following are ok because they're parameters
              integer, dimension(*), parameter :: param = [1, 2, 3]
              character(*), dimension(*), parameter :: param_char = ['hello']
            end subroutine assumed_size_dimension
            ",
        );
        let expected: Vec<Violation> = [
            (3, 27, 3, 28, "array"),
            (4, 27, 4, 28, "l"),
            (4, 36, 4, 37, "p"),
            (6, 30, 6, 31, "options"),
            (7, 24, 7, 25, "thing"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, variable)| {
            Violation::from_start_end_line_col(
                format!("'{variable}' has assumed size"),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = AssumedSize::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_assumed_size_character_intent() -> anyhow::Result<()> {
        // test case from stylist
        let source = test_file(
            "
            program cases
              ! A char array outside a function or subroutine, no exception
              character (*) :: autochar_glob
            contains
              subroutine char_input(autochar_in, autochar_inout, autochar_out, fixedchar)
                ! A char array with proper intent, no exception
                character(*), intent(in)       :: autochar_in
                ! A char array with disallowed intent, exception
                character(*), intent(inout)    :: autochar_inout
                ! A char array with disallowed intent, exception
                character(len=*), intent(out)  :: autochar_out
                ! A char array not passed as a parameter, no exception
                character(*)                   :: autochar_var
                ! A char array with fixed length, no exception
                character(len=10), intent(out) :: fixedchar
                ! A declaration with non-intent attribute, no exception
                character(len=*), parameter :: alt_attr = 'sample'
              end subroutine char_input
            end program cases
            ",
        );
        let expected: Vec<Violation> = [
            (3, 13, 3, 14, "autochar_glob"),
            (9, 14, 9, 15, "autochar_inout"),
            (11, 18, 11, 19, "autochar_out"),
            (13, 14, 13, 15, "autochar_var"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, variable)| {
            Violation::from_start_end_line_col(
                format!("character '{variable}' has assumed size but does not have `intent(in)`"),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = AssumedSizeCharacterIntent::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_deprecated_assumed_size_character() -> anyhow::Result<()> {
        // test case from stylist
        let source = test_file(
            "
            program cases
            contains
              subroutine char_input(a, b, c, d, e, f)
                character * ( * ), intent(in) :: a
                character*(*), intent(in) :: b
                character*(len=*), intent(in) :: c
                character*(3), intent(in) :: d
                character*(MAX_LEN), intent(in) :: e
                ! these are ok
                character(*, kind) :: f
                character(len=*, kind=4) :: g
              end subroutine char_input
            end program cases
            ",
        );
        let expected: Vec<Violation> = [
            (4, 14, 4, 15, "a"),
            (5, 13, 5, 14, "b"),
            (6, 13, 6, 14, "c"),
            (7, 13, 7, 14, "d"),
            (8, 13, 8, 14, "e"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, variable)| {
            Violation::from_start_end_line_col(
                format!("character '{variable}' uses deprecated syntax for assumed size"),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = DeprecatedAssumedSizeCharacter::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
