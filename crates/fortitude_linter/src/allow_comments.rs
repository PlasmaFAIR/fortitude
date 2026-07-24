use crate::CheckContext;
use crate::ast::FortitudeNode;
use crate::rule_redirects::get_redirect_target;
use crate::rules::Rule;
use crate::rules::fortitude::allow_comments::{
    DisabledAllowComment, DuplicatedAllowComment, InvalidRuleCodeOrName, RedirectedAllowComment,
    UnusedAllowComment,
};
use crate::traits::TextRanged;

use crate::diagnostics::{Diagnostic, Edit, Fix};
use itertools::Itertools;
use lazy_regex::{regex, regex_captures};
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

impl<'a> TextRanged for &Code<'a> {
    fn textrange(&self) -> TextRange {
        self.loc
    }
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
    let range = {
        let next_node = node.next_named_sibling()?;
        let start_byte = next_node.start_textsize();
        let end_byte = next_node.end_textsize();

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
    };

    // Partition the found selectors into valid and invalid
    let rule_regex = regex!(r#"\w[-\w\d]*"#);
    // 8 from length of "! allow("
    let comment_start_offset = node.start_textsize() + TextSize::new(8);
    for rule in rule_regex.find_iter(allow_comment) {
        let start = comment_start_offset + TextSize::try_from(rule.start()).unwrap();
        let end = comment_start_offset + TextSize::try_from(rule.end()).unwrap();
        let loc = TextRange::new(start, end);
        let code = rule.as_str();
        let redirect = get_redirect_target(code).unwrap_or(code);
        let rule = Rule::from_code(redirect).or(Rule::from_str(redirect)).ok();

        codes.push(Code { code, rule, loc });
    }

    Some(AllowComment {
        codes,
        range,
        node: *node,
    })
}

/// Check allow comments, raise applicable violations, and ignore allowed diagnostics
///
/// Returns list of indices of allowed violations
pub(crate) fn check_allow_comments(
    diagnostics: &mut Vec<Diagnostic>,
    allow_comments: &[AllowComment],
    context: &CheckContext,
) -> Vec<usize> {
    // Indices of diagnostics that were ignored by a `noqa` directive.
    let mut ignored_diagnostics = vec![];

    let mut used_codes = FxHashSet::default();

    // Remove any ignored diagnostics
    'outer: for (index, diagnostic) in diagnostics.iter().enumerate() {
        for allow in allow_comments {
            for code in &allow.codes {
                if let Some(rule) = code.rule
                    && code.rule == diagnostic.rule()
                    && allow
                        .range
                        .contains_range(diagnostic.range().unwrap_or_default())
                {
                    used_codes.insert(rule);
                    ignored_diagnostics.push(index);
                    // We've ignored this diagnostic, so no point
                    // checking the other allow comments!
                    continue 'outer;
                }
            }
        }
    }

    for comment in allow_comments {
        let mut seen_codes = FxHashSet::default();

        for code in &comment.codes {
            let redirect = get_redirect_target(code.code);
            if context.is_rule_enabled(Rule::RedirectedAllowComment)
                && let Some(redirect) = redirect
            {
                let rule = Rule::from_code(redirect).unwrap();
                let new_code = rule.noqa_code().to_string();
                let new_name = rule.as_ref().to_string();
                let edit = Edit::replacement(new_name.clone(), code.loc.start(), code.loc.end());
                diagnostics.push(
                    context
                        .create_diagnostic(
                            RedirectedAllowComment {
                                original: code.code.to_string(),
                                redirect: redirect.to_string(),
                                new_code,
                                new_name,
                            },
                            code,
                        )
                        .with_fix(Fix::safe_edit(edit)),
                );
            }

            let rule = code.code.to_string();
            let edit = remove_code_from_allow_comment(comment, code, context.source_file());

            let diagnostic = match code.rule {
                None => context.create_diagnostic_if_enabled(InvalidRuleCodeOrName { rule }, code),
                Some(rule_code) => {
                    let used = used_codes.contains(&rule_code);
                    let enabled = context.is_rule_enabled(rule_code);
                    if !seen_codes.insert(rule_code) {
                        context.create_diagnostic_if_enabled(DuplicatedAllowComment { rule }, code)
                    } else if !enabled {
                        context.create_diagnostic_if_enabled(DisabledAllowComment { rule }, code)
                    } else if !used && enabled {
                        context.create_diagnostic_if_enabled(UnusedAllowComment { rule }, code)
                    } else {
                        None
                    }
                }
            };

            if let Some(diagnostic) = diagnostic {
                diagnostics.push(diagnostic.with_fix(Fix::safe_edit(edit)));
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
            .edit_replacement(file, format!("! allow({remaining_codes})"))
    }
}
