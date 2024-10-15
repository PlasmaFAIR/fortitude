use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

pub struct UnnamedEndStatement {}

impl Rule for UnnamedEndStatement {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        End statements should specify what kind of construct they're ending, and the
        name of that construct. For example, prefer this:

        ```
        module mymodule
          ...
        end module mymodule
        ```

        To this:

        ```
        module mymodule
          ...
        end
        ```

        Or this:

        ```
        module mymodule
          ...
        end module
        ```

        Similar rules apply for many other Fortran statements
        "
    }
}

/// Maps declaration kinds to its name and the kind of the declaration statement node
fn map_declaration(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "module" => ("module", "module_statement"),
        "submodule" => ("submodule", "submodule_statement"),
        "program" => ("program", "program_statement"),
        "function" => ("function", "function_statement"),
        "subroutine" => ("subroutine", "subroutine_statement"),
        "module_procedure" => ("procedure", "module_procedure_statement"),
        "derived_type_definition" => ("type", "derived_type_statement"),
        _ => unreachable!("Invalid entrypoint for AbbreviatedEndStatement"),
    }
}

impl ASTRule for UnnamedEndStatement {
    fn check<'a>(&self, node: &'a Node, src: &'a SourceFile) -> Option<Vec<Violation>> {
        // TODO Also check for optionally labelled constructs like 'do' or 'select'

        // If end node is named, move on.
        // Not catching incorrect end statement name here, as the compiler should
        // do that for us.
        if node.child_with_name("name").is_some() {
            return None;
        }

        let declaration = node.parent()?;
        let (statement, statement_kind) = map_declaration(declaration.kind());
        let statement_node = declaration.child_with_name(statement_kind)?;
        let name_kind = match statement_kind {
            "derived_type_statement" => "type_name",
            _ => "name",
        };
        let name = statement_node
            .child_with_name(name_kind)?
            .to_text(src.source_text())?;
        let msg = format!("end statement should read 'end {statement} {name}'");
        some_vec![Violation::from_node(msg, node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec![
            "end_module_statement",
            "end_submodule_statement",
            "end_program_statement",
            "end_function_statement",
            "end_subroutine_statement",
            "end_module_procedure_statement",
            "end_type_statement",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unnamed_end_statement() -> anyhow::Result<()> {
        let source = test_file(
            "
            module mymod1
              implicit none
              type mytype
                integer :: x
              end type                      ! catch this
            contains
              subroutine mysub1()
                write (*,*) 'hello world'
              end subroutine                ! catch this
              subroutine mysub2()
                write (*,*) 'hello world'
              end subroutine mysub2         ! ignore this
            end                             ! catch this
            module mymod2
              implicit none
              type mytype
                integer :: x
              end type mytype               ! ignore this
            contains
              integer function myfunc1()
                myfunc1 = 1
              end function                  ! catch this
              integer function myfunc2()
                myfunc2 = 1
              end function myfunc2          ! ignore this
            end module                      ! catch this
            module mymod3
              interface
                module function foo() result(x)
                  integer :: x
                end function foo            ! ignore this
                module function bar() result(x)
                  integer :: x
                end function bar            ! ignore this
                module function baz() result(x)
                  integer :: x
                end function baz            ! ignore this
              end interface
            end module mymod3
            submodule (mymod3) mysub1
            contains
              module procedure foo
                x = 1
              end procedure                 ! catch this
            end                             ! catch this
            submodule (mymod3) mysub2
            contains
              module procedure bar
                x = 1
              end procedure bar             ! ignore this
            end submodule                   ! catch this
            submodule (mymod3) mysub3
            contains
              module procedure baz
                x = 1
              end procedure baz             ! ignore this
            end submodule mysub3            ! ignore this
            program myprog
              implicit none
              write (*,*) 'hello world'
            end                             ! catch this
            ",
        );
        let expected: Vec<Violation> = [
            (5, 2, 5, 32, "type", "mytype"),
            (9, 2, 9, 32, "subroutine", "mysub1"),
            (13, 0, 13, 32, "module", "mymod1"),
            (22, 2, 22, 32, "function", "myfunc1"),
            (26, 0, 26, 32, "module", "mymod2"),
            (44, 2, 44, 32, "procedure", "foo"),
            (45, 0, 45, 32, "submodule", "mysub1"),
            (51, 0, 51, 32, "submodule", "mysub2"),
            (61, 0, 61, 32, "program", "myprog"),
        ]
        .iter()
        .map(
            |(start_line, start_col, end_line, end_col, statement, name)| {
                Violation::from_start_end_line_col(
                    format!("end statement should read 'end {statement} {name}'"),
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            },
        )
        .collect();
        let rule = UnnamedEndStatement::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
