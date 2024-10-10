use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use tree_sitter::Node;

pub struct AbbreviatedEndStatement {}

impl Rule for AbbreviatedEndStatement {
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

        Similar rules apply for submodules, programs, functions, subroutines, and
        derived types.
        "
    }
}

fn map_start_statement(kind: &str) -> &'static str {
    match kind {
        "module" => "module_statement",
        "submodule" => "submodule_statement",
        "program" => "program_statement",
        "function" => "function_statement",
        "subroutine" => "subroutine_statement",
        "module_procedure" => "module_procedure_statement",
        "derived_type_definition" => "derived_type_statement",
        _ => unreachable!("Invalid entrypoint for AbbreviatedEndStatement"),
    }
}

fn map_end_statement(kind: &str) -> &'static str {
    match kind {
        "module" => "end_module_statement",
        "submodule" => "end_submodule_statement",
        "program" => "end_program_statement",
        "function" => "end_function_statement",
        "subroutine" => "end_subroutine_statement",
        "module_procedure" => "end_module_procedure_statement",
        "derived_type_definition" => "end_type_statement",
        _ => unreachable!("Invalid entrypoint for AbbreviatedEndStatement"),
    }
}

fn map_statement_type(kind: &str) -> &'static str {
    match kind {
        "module" => "module",
        "submodule" => "submodule",
        "program" => "program",
        "function" => "function",
        "subroutine" => "subroutine",
        "module_procedure" => "procedure",
        "derived_type_definition" => "type",
        _ => unreachable!("Invalid entrypoint for AbbreviatedEndStatement"),
    }
}

impl ASTRule for AbbreviatedEndStatement {
    fn check<'a>(&self, node: &'a Node, src: &'a str) -> Option<Vec<Violation>> {
        let kind = node.kind();
        let end_kind = map_end_statement(kind);
        let end_node = node.child_with_name(end_kind)?;

        // If end node is named, move on.
        // Not catching incorrect end statement name here, as the compiler should
        // do that for us.
        if end_node.child_with_name("name").is_some() {
            return None;
        }

        let start_kind = map_start_statement(kind);
        let start_node = node.child_with_name(start_kind)?;
        let name_kind = match kind {
            "derived_type_definition" => "type_name",
            _ => "name",
        };
        let name = start_node.child_with_name(name_kind)?.to_text(src)?;
        let statement = map_statement_type(kind);
        let msg = format!("end statement should read 'end {statement} {name}'");
        some_vec![Violation::from_node(msg, &end_node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec![
            "module",
            "submodule",
            "program",
            "function",
            "subroutine",
            "module_procedure",
            "derived_type_definition",
        ]
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
    fn test_abbreviated_end_statement() -> anyhow::Result<()> {
        let source = dedent(
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
            (14, 1, "module", "mymod1"),
            (6, 3, "type", "mytype"),
            (10, 3, "subroutine", "mysub1"),
            (27, 1, "module", "mymod2"),
            (23, 3, "function", "myfunc1"),
            (46, 1, "submodule", "mysub1"),
            (45, 3, "procedure", "foo"),
            (52, 1, "submodule", "mysub2"),
            (62, 1, "program", "myprog"),
        ]
        .iter()
        .map(|(line, col, statement, name)| {
            let msg = format!("end statement should read 'end {statement} {name}'");
            violation!(msg, *line, *col)
        })
        .collect();
        let rule = AbbreviatedEndStatement::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
