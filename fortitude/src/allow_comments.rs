use crate::ast::FortitudeNode;
use crate::registry::AsRule;
use crate::rule_redirects::get_redirect_target;
use crate::rule_table::RuleTable;
use crate::rules::fortitude::allow_comments::{
    DisabledAllowComment, DuplicatedAllowComment, InvalidRuleCodeOrName, RedirectedAllowComment,
    UnusedAllowComment,
};
use crate::rules::Rule;

use lazy_regex::{regex, regex_captures};
use ruff_diagnostics::{Diagnostic, Edit, Fix};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use rustc_hash::FxHashSet;
use std::str::FromStr;
use tree_sitter::Node;

/// A single allowed rule and the range it applies to
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AllowComment<'a> {
    // The original rule/code/category in the comment
    pub code: &'a str,
    // Resolved rule
    pub rule: Option<Rule>,
    // The range the comment applies to
    pub range: TextRange,
    // The location of the comment
    pub loc: TextRange,
}

/// If this node is an `allow` comment, get all the rules allowed on the next line
pub fn gather_allow_comments<'a>(
    node: &Node,
    file: &'a SourceFile,
) -> Option<Vec<AllowComment<'a>>> {
    if node.kind() != "comment" {
        return None;
    }

    let mut allow_comments = Vec::new();

    if let Some((_, allow_comment)) = regex_captures!(
        r#"! allow\((.*)\)\s*"#,
        node.to_text(file.source_text()).unwrap()
    ) {
        let range = if let Some(next_node) = node.next_named_sibling() {
            let start_byte = TextSize::try_from(next_node.start_byte()).unwrap();
            let end_byte = TextSize::try_from(next_node.end_byte()).unwrap();

            // This covers the next statement _upto_ the end of the
            // line that it _ends_ on -- i.e. including trailing
            // whitespace and other statements. This might have weird
            // edge cases.
            let src = file.to_source_code();
            let start_index = src.line_index(start_byte);
            let end_index = src.line_index(end_byte);
            let start_line = src.line_start(start_index);
            let end_line = src.line_end(end_index);

            TextRange::new(start_line, end_line)
        } else {
            return None;
        };

        // Partition the found selectors into valid and invalid
        let rule_regex = regex!(r#"\w[-\w\d]*"#);
        // 8 from length of "! allow("
        let comment_start_offset =
            TextSize::try_from(node.start_byte()).unwrap() + TextSize::new(8);
        for rule in rule_regex.find_iter(allow_comment) {
            let start = comment_start_offset + TextSize::try_from(rule.start()).unwrap();
            let end = comment_start_offset + TextSize::try_from(rule.end()).unwrap();
            let loc = TextRange::new(start, end);
            let code = rule.as_str();
            let redirect = get_redirect_target(code).unwrap_or(code);
            let rule = match Rule::from_code(redirect).or(Rule::from_str(redirect)) {
                Ok(rule) => Some(rule),
                Err(_) => None,
            };

            allow_comments.push(AllowComment {
                code,
                rule,
                range,
                loc,
            });
        }
    }

    Some(allow_comments)
}

/// Check allow comments, raise applicable violations, and ignore allowed diagnostics
pub fn check_allow_comments(
    diagnostics: &mut Vec<Diagnostic>,
    allow_comments: &[AllowComment],
    rules: &RuleTable,
) -> Vec<usize> {
    // Indices of diagnostics that were ignored by a `noqa` directive.
    let mut ignored_diagnostics = vec![];

    let mut used_codes = FxHashSet::default();

    // Remove any ignored diagnostics
    'outer: for (index, diagnostic) in diagnostics.iter().enumerate() {
        for allow in allow_comments {
            if let Some(rule) = allow.rule {
                if rule == diagnostic.kind.rule() && allow.range.contains_range(diagnostic.range) {
                    used_codes.insert(rule);
                    ignored_diagnostics.push(index);
                    // We've ignored this diagnostic, so no point
                    // checking the other allow comments!
                    continue 'outer;
                };
            }
        }
    }

    let mut seen_codes = FxHashSet::default();

    for comment in allow_comments {
        let redirect = get_redirect_target(comment.code);
        if rules.enabled(Rule::RedirectedAllowComment) {
            if let Some(redirect) = redirect {
                let new_name = Rule::from_code(redirect).unwrap().as_ref().to_string();
                let edit =
                    Edit::replacement(new_name.clone(), comment.loc.start(), comment.loc.end());
                diagnostics.push(
                    Diagnostic::new(
                        RedirectedAllowComment {
                            original: comment.code.to_string(),
                            new_code: redirect.to_string(),
                            new_name,
                        },
                        comment.loc,
                    )
                    .with_fix(Fix::safe_edit(edit)),
                );
            }
        }

        let code = redirect.unwrap_or(comment.code);

        match comment.rule {
            None => diagnostics.push(Diagnostic::new(
                InvalidRuleCodeOrName {
                    rule: code.to_string(),
                },
                comment.loc,
            )),
            Some(rule) => {
                if !seen_codes.insert(rule) {
                    if rules.enabled(Rule::DuplicatedAllowComment) {
                        diagnostics.push(Diagnostic::new(
                            DuplicatedAllowComment {
                                rule: comment.code.to_string(),
                            },
                            comment.loc,
                        ));
                    }
                } else if !used_codes.contains(&rule) {
                    if rules.enabled(rule) {
                        if rules.enabled(Rule::UnusedAllowComment) {
                            diagnostics.push(Diagnostic::new(
                                UnusedAllowComment {
                                    rule: code.to_string(),
                                },
                                comment.loc,
                            ));
                        }
                    } else if rules.enabled(Rule::DisabledAllowComment) {
                        diagnostics.push(Diagnostic::new(
                            DisabledAllowComment {
                                rule: code.to_string(),
                            },
                            comment.loc,
                        ));
                    }
                }
            }
        }
    }

    ignored_diagnostics.sort_unstable();
    ignored_diagnostics
}
