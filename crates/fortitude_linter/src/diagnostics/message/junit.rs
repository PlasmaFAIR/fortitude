// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::Path;

use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite, XmlString};

use crate::diagnostics::message::grouped::DiagnosticWithLocation;
use crate::diagnostics::{Diagnostic, SecondaryCode};

use super::grouped::group_diagnostics_by_filename;

/// Print diagnostics as a JUnit-style XML report.
///
/// See [`junit.xsd`] for the specification in the JUnit repository and an annotated [version]
/// linked from the [`quick_junit`] docs.
///
/// [`junit.xsd`]: https://github.com/junit-team/junit-framework/blob/2870b7d8fd5bf7c1efe489d3991d3ed3900e82bb/platform-tests/src/test/resources/jenkins-junit.xsd
/// [version]: https://llg.cubic.org/docs/junit/
/// [`quick_junit`]: https://docs.rs/quick-junit/latest/quick_junit/
pub(super) struct JunitRenderer {}

impl JunitRenderer {
    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        let package = "org.plasmafair.fortitude";
        let mut report = Report::new("fortitude");

        if diagnostics.is_empty() {
            let mut test_suite = TestSuite::new("fortitude");
            test_suite
                .extra
                .insert(XmlString::new("package"), XmlString::new(package));
            let mut case = TestCase::new("No errors found", TestCaseStatus::success());
            case.set_classname("fortitude");
            test_suite.add_test_case(case);
            report.add_test_suite(test_suite);
        } else {
            for (filename, diagnostics) in group_diagnostics_by_filename(diagnostics) {
                let mut test_suite = TestSuite::new(&filename);
                test_suite
                    .extra
                    .insert(XmlString::new("package"), XmlString::new(package));

                let classname = Path::new(&filename).with_extension("");

                for diagnostic in diagnostics {
                    let DiagnosticWithLocation {
                        diagnostic,
                        start_location: location,
                    } = diagnostic;

                    let code = diagnostic
                        .secondary_code()
                        .map_or_else(|| diagnostic.name(), SecondaryCode::as_str);
                    let mut status = TestCaseStatus::non_success(NonSuccessKind::Failure);
                    status.set_message(diagnostic.concise_message().to_str());

                    status.set_description(format!(
                        "line {row}, col {col}, {body}",
                        row = location.line,
                        col = location.column,
                        body = diagnostic.concise_message()
                    ));

                    let mut case =
                        TestCase::new(format!("org.plasmafair.fortitude.{code}"), status);
                    case.set_classname(classname.to_str().unwrap_or(&filename));

                    case.extra.insert(
                        XmlString::new("line"),
                        XmlString::new(location.line.to_string()),
                    );
                    case.extra.insert(
                        XmlString::new("column"),
                        XmlString::new(location.column.to_string()),
                    );

                    test_suite.add_test_case(case);
                }
                report.add_test_suite(test_suite);
            }
        }

        let adapter = FmtAdapter { fmt: f };
        report.serialize(adapter).map_err(|_| std::fmt::Error)
    }
}

struct FmtAdapter<'a> {
    fmt: &'a mut dyn std::fmt::Write,
}

impl std::io::Write for FmtAdapter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fmt
            .write_str(std::str::from_utf8(buf).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid UTF-8 in JUnit report",
                )
            })?)
            .map_err(std::io::Error::other)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.fmt.write_fmt(args).map_err(std::io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{
        OutputFormat,
        message::tests::{create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Junit);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Junit);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }
}
