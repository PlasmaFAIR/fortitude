use crate::ast::FortitudeNode;
use crate::registry::AsRule;
use crate::rule_redirects::get_redirect_target;
use crate::rule_table::RuleTable;
use crate::rules::fortitude::allow_comments::{
    DisabledAllowComment, DuplicatedAllowComment, InvalidRuleCodeOrName, RedirectedAllowComment,
    UnusedAllowComment,
};
use crate::rules::Rule;

use itertools::Itertools;
use lazy_regex::{regex, regex_captures};
use ruff_diagnostics::{Diagnostic, Edit, Fix};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use rustc_hash::FxHashSet;
use std::str::FromStr;
use tree_sitter::Node;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Code<'a> {
    // The original rule/code/category in the comment
    pub code: &'a str,
    // Resolved rule
    pub rule: Option<Rule>,
    // The location of the code
    pub loc: TextRange,
}

/// A single allowed rule and the range it applies to
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AllowComment<'a, 'b> {
    // Codes in the allow comment
    pub codes: Vec<Code<'a>>,
    // The range the comment applies to
    pub range: TextRange,
    // The comment node
    pub node: Node<'b>,
}

/// If this node is an `allow` comment, get all the rules allowed on the next line
pub fn gather_allow_comments<'a, 'b>(
    node: &Node<'b>,
    file: &'a SourceFile,
) -> Option<AllowComment<'a, 'b>> {
    if node.kind() != "comment" {
        return None;
    }

    let mut codes = Vec::new();

    let (_, allow_comment) = regex_captures!(
        r#"! allow\((.*)\)\s*"#,
        node.to_text(file.source_text()).unwrap()
    )?;
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
    let comment_start_offset = TextSize::try_from(node.start_byte()).unwrap() + TextSize::new(8);
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

        codes.push(Code { code, rule, loc });
    }

    Some(AllowComment {
        codes,
        range,
        node: *node,
    })
}

/// Check allow comments, raise applicable violations, and ignore allowed diagnostics
pub fn check_allow_comments(
    diagnostics: &mut Vec<Diagnostic>,
    allow_comments: &[AllowComment],
    rules: &RuleTable,
    file: &SourceFile,
) -> Vec<usize> {
    // Indices of diagnostics that were ignored by a `noqa` directive.
    let mut ignored_diagnostics = vec![];

    let mut used_codes = FxHashSet::default();

    // Remove any ignored diagnostics
    'outer: for (index, diagnostic) in diagnostics.iter().enumerate() {
        for allow in allow_comments {
            for code in &allow.codes {
                if let Some(rule) = code.rule {
                    if rule == diagnostic.kind.rule()
                        && allow.range.contains_range(diagnostic.range)
                    {
                        used_codes.insert(rule);
                        ignored_diagnostics.push(index);
                        // We've ignored this diagnostic, so no point
                        // checking the other allow comments!
                        continue 'outer;
                    };
                }
            }
        }
    }

    for comment in allow_comments {
        let mut seen_codes = FxHashSet::default();

        for code in &comment.codes {
            let redirect = get_redirect_target(code.code);
            if rules.enabled(Rule::RedirectedAllowComment) {
                if let Some(redirect) = redirect {
                    let new_name = Rule::from_code(redirect).unwrap().as_ref().to_string();
                    let edit =
                        Edit::replacement(new_name.clone(), code.loc.start(), code.loc.end());
                    diagnostics.push(
                        Diagnostic::new(
                            RedirectedAllowComment {
                                original: code.code.to_string(),
                                new_code: redirect.to_string(),
                                new_name,
                            },
                            code.loc,
                        )
                        .with_fix(Fix::safe_edit(edit)),
                    );
                }
            }

            let rule_str = code.code.to_string();
            let edit = remove_code_from_allow_comment(comment, code, file);

            match code.rule {
                None => {
                    if rules.enabled(Rule::InvalidRuleCodeOrName) {
                        diagnostics.push(
                            Diagnostic::new(InvalidRuleCodeOrName { rule: rule_str }, code.loc)
                                .with_fix(Fix::safe_edit(edit)),
                        );
                    }
                }
                Some(rule) => {
                    let used = used_codes.contains(&rule);
                    let enabled = rules.enabled(rule);
                    if !seen_codes.insert(rule) && rules.enabled(Rule::DuplicatedAllowComment) {
                        diagnostics.push(
                            Diagnostic::new(DuplicatedAllowComment { rule: rule_str }, code.loc)
                                .with_fix(Fix::safe_edit(edit)),
                        );
                    } else if !enabled && rules.enabled(Rule::DisabledAllowComment) {
                        diagnostics.push(
                            Diagnostic::new(DisabledAllowComment { rule: rule_str }, code.loc)
                                .with_fix(Fix::safe_edit(edit)),
                        );
                    } else if !used && enabled && rules.enabled(Rule::UnusedAllowComment) {
                        diagnostics.push(
                            Diagnostic::new(UnusedAllowComment { rule: rule_str }, code.loc)
                                .with_fix(Fix::safe_edit(edit)),
                        );
                    }
                }
            }
        }
    }

    ignored_diagnostics.sort_unstable();
    ignored_diagnostics
}

fn remove_code_from_allow_comment(
    comment: &AllowComment,
    code_to_remove: &Code,
    file: &SourceFile,
) -> Edit {
    let remaining_codes = comment
        .codes
        .iter()
        .filter(|code| *code != code_to_remove)
        .map(|code| code.code)
        .join(", ");

    if remaining_codes.is_empty() {
        comment.node.edit_delete(file)
    } else {
        comment
            .node
            .edit_replacement(file, format!("! allow({})", remaining_codes))
    }
}
