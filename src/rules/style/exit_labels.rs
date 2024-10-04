use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use tree_sitter::Node;

pub struct MissingExitOrCycleLabel {}

impl Rule for MissingExitOrCycleLabel {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        When using `exit` or `cycle` in a named `do` loop, the `exit`/`cycle` statement
        should use the loop name

        ```
        name: do
          exit name
        end do name
        ```

        Using named loops is particularly useful for nested or complicated loops, as it
        helps the reader keep track of the flow of logic. It's also the only way to `exit`
        or `cycle` outer loops from within inner ones.
        "
    }
}

impl ASTRule for MissingExitOrCycleLabel {
    fn check<'a>(&self, node: &'a Node, src: &'a str) -> Option<Vec<Violation>> {
        // Skip unlabelled loops
        let label = node
            .child_with_name("block_label_start_expression")?
            .to_text(src)?
            .trim_end_matches(':');

        let violations: Vec<Violation> = node
            .named_descendants_except(["do_loop_statement"])
            .filter(|node| node.kind() == "keyword_statement")
            .map(|stmt| (stmt, stmt.to_text(src).unwrap_or_default().to_lowercase()))
            .filter(|(_, name)| name == "exit" || name == "cycle")
            .map(|(stmt, name)| {
                let msg = format!("'{name}' statement in named 'do' loop missing label '{label}'");
                Violation::from_node(msg, &stmt)
            })
            .collect();

        Some(violations)
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["do_loop_statement"]
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
    fn test_missing_exit_label() -> anyhow::Result<()> {
        let source = dedent(
            "
            program test
              label1: do
                if (.true.) then
                  EXIT
                end if
              end do label1

              label2: do
                if (.true.) exit
              end do label2

              label3: do
                exit label3
              end do label3

              label4: do
                if (.true.) exit label4
              end do label4

              label5: do i = 1, 2
                do j = 1, 2  ! unnamed inner loop: currently doesn't warn
                  cycle
                end do
              end do label5

              label6: do i = 1, 2
                inner: do j = 1, 2
                  if (.true.) CYCLE ! named inner loop: warns on inner loop
                end do inner
              end do label6

              label7: do
                cycle label7
              end do label7

              label8: do
                if (.true.) cycle label8
              end do label8

              do
                ! Don't warn on unnamed loops
                exit
              end do
            end program test
            ",
        );
        let expected: Vec<Violation> = [
            (5, 7, "exit", "label1"),
            (10, 17, "exit", "label2"),
            (29, 19, "cycle", "inner"),
        ]
        .iter()
        .map(|(line, col, stmt_kind, label)| {
            let msg = format!("'{stmt_kind}' statement in named 'do' loop missing label '{label}'");
            violation!(msg, *line, *col)
        })
        .collect();
        let rule = MissingExitOrCycleLabel::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
