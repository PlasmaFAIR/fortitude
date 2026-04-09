pub mod complexity;
pub(crate) mod double_colon_in_decl;
pub(crate) mod end_statements;
pub(crate) mod file_contents;
pub(crate) mod file_extensions;
pub(crate) mod functions;
pub(crate) mod implicit_none;
pub mod inconsistent_dimension;
pub mod keywords;
pub mod line_length;
pub(crate) mod semicolons;
pub mod strings;
pub(crate) mod use_statement;
pub mod useless_return;
pub(crate) mod whitespace;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::rules::style::inconsistent_dimension::settings::PreferAttribute;
    use crate::rules::style::{complexity, inconsistent_dimension, keywords, line_length, strings};
    use crate::settings::CheckSettings;
    use crate::test::test_path;

    use super::strings::settings::Quote;

    use std::borrow::Cow;

    pub(crate) trait ShowNonprinting {
        fn show_nonprinting(&self) -> Cow<'_, str>;
    }

    macro_rules! impl_show_nonprinting {
        ($(($from:expr, $to:expr)),+) => {
            impl ShowNonprinting for str {
                fn show_nonprinting(&self) -> Cow<'_, str> {
                    if self.find(&[$($from),*][..]).is_some() {
                        Cow::Owned(
                            self.$(replace($from, $to)).*
                        )
                    } else {
                        Cow::Borrowed(self)
                    }
                }
            }
        };
    }

    impl_show_nonprinting!(
        ('\x07', "␇"),
        ('\x08', "␈"),
        ('\x1b', "␛"),
        ('\x7f', "␡"),
        ('\x0A', "␊\n"),
        ('\x0D', "␍")
    );

    #[test_case(Rule::SuperfluousElseReturn, Path::new("S252.f90"))]
    #[test_case(Rule::SuperfluousElseCycle, Path::new("S253.f90"))]
    #[test_case(Rule::SuperfluousElseExit, Path::new("S254.f90"))]
    #[test_case(Rule::SuperfluousElseStop, Path::new("S255.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics.show_nonprinting());
        Ok(())
    }
}
