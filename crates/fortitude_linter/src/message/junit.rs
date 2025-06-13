// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::path::Path;

use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite, XmlString};

use crate::message::{group_messages_by_filename, Emitter, MessageWithLocation};

use super::DiagnosticMessage;

#[derive(Default)]
pub struct JunitEmitter;

impl Emitter for JunitEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        let mut report = Report::new("fortitude");

        if messages.is_empty() {
            let mut test_suite = TestSuite::new("fortitude");
            test_suite.extra.insert(
                XmlString::new("package"),
                XmlString::new("org.plasmafair.fortitude"),
            );
            let mut case = TestCase::new("No errors found", TestCaseStatus::success());
            case.set_classname("fortitude");
            test_suite.add_test_case(case);
            report.add_test_suite(test_suite);
        } else {
            for (filename, messages) in group_messages_by_filename(messages) {
                let mut test_suite = TestSuite::new(filename);
                test_suite.extra.insert(
                    XmlString::new("package"),
                    XmlString::new("org.plasmafair.fortitude"),
                );

                for message in messages {
                    let MessageWithLocation {
                        message,
                        start_location,
                    } = message;
                    let mut status = TestCaseStatus::non_success(NonSuccessKind::Failure);
                    status.set_message(message.body());
                    let location = start_location;

                    status.set_description(format!(
                        "line {row}, col {col}, {body}",
                        row = location.row,
                        col = location.column,
                        body = message.body()
                    ));
                    let mut case = TestCase::new(
                        if let Some(rule) = message.rule() {
                            format!("org.plasmafair.fortitude.{}", rule.noqa_code())
                        } else {
                            "org.plasmafair.fortitude".to_string()
                        },
                        status,
                    );
                    let file_path = Path::new(filename);
                    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
                    let classname = file_path.parent().unwrap().join(file_stem);
                    case.set_classname(classname.to_str().unwrap());
                    case.extra.insert(
                        XmlString::new("line"),
                        XmlString::new(location.row.to_string()),
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

        report.serialize(writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::JunitEmitter;

    #[test]
    fn output() {
        let mut emitter = JunitEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
