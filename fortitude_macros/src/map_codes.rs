// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized, parse::Parse, spanned::Spanned, Attribute, Error, Expr, ExprCall, ExprMatch,
    Ident, ItemFn, LitStr, Pat, Path, Stmt, Token,
};

use crate::rule_code_prefix::{get_prefix_ident, intersection_all};

/// A rule entry in the big match statement such a
/// `(Pycodestyle, "E112") => (RuleGroup::Preview, rules::pycodestyle::rules::logical_lines::NoIndentedBlock),`
#[derive(Clone)]
struct RuleMeta {
    /// The actual name of the rule, e.g., `NoIndentedBlock`.
    name: Ident,
    /// The category associated with the rule, e.g., `Typing`.
    category: Ident,
    /// The code associated with the rule, e.g., `"E112"`.
    code: LitStr,
    /// The kind of checker, e.g. `Text`
    kind: Path,
    /// The rule group identifier, e.g., `RuleGroup::Preview`.
    group: Path,
    /// The path to the struct implementing the rule, e.g.
    /// `rules::pycodestyle::rules::logical_lines::NoIndentedBlock`
    path: Path,
    /// The rule attributes, e.g. for feature gates
    attrs: Vec<Attribute>,
}

pub(crate) fn map_codes(func: &ItemFn) -> syn::Result<TokenStream> {
    // Check that the `func` is in the form we expect
    let Some(last_stmt) = func.block.stmts.last() else {
        return Err(Error::new(
            func.block.span(),
            "expected body to end in an expression",
        ));
    };
    let Stmt::Expr(
        Expr::Call(ExprCall {
            args: some_args, ..
        }),
        _,
    ) = last_stmt
    else {
        return Err(Error::new(
            last_stmt.span(),
            "expected last expression to be `Some(match (..) { .. })`",
        ));
    };
    let mut some_args = some_args.into_iter();
    // `arms` are the set of `match` arms, each defining one rule
    let (Some(Expr::Match(ExprMatch { arms, .. })), None) = (some_args.next(), some_args.next())
    else {
        return Err(Error::new(
            last_stmt.span(),
            "expected last expression to be `Some(match (..) { .. })`",
        ));
    };

    // Map from: category (e.g., `Typing`) to rule code (e.g.,`"002"`) to rule data (e.g.,
    // `(Rule::UnaryPrefixIncrement, RuleGroup::Stable, vec![])`).
    let mut category_to_rules: BTreeMap<Ident, BTreeMap<String, RuleMeta>> = BTreeMap::new();

    for arm in arms {
        if matches!(arm.pat, Pat::Wild(..)) {
            break;
        }

        let rule = syn::parse::<RuleMeta>(arm.into_token_stream().into())?;
        category_to_rules
            .entry(rule.category.clone())
            .or_default()
            .insert(rule.code.value(), rule);
    }

    let category_idents: Vec<_> = category_to_rules.keys().collect();

    let all_rules = category_to_rules.values().flat_map(BTreeMap::values);
    let mut output = register_rules(all_rules);

    // Create an enum that lets us map from `Category` variants to
    // their prefixes, which may include subcategories (not currently
    // used, but used in ruff)
    output.extend(quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RuleCodePrefix {
            #(#category_idents(#category_idents),)*
        }

        impl RuleCodePrefix {
            pub fn category(&self) -> &'static Category {
                match self {
                    #(Self::#category_idents(..) => &Category::#category_idents,)*
                }
            }

            pub fn short_code(&self) -> &'static str {
                match self {
                    #(Self::#category_idents(code) => code.into(),)*
                }
            }
        }
    });

    // Conversion functions from `Category` to `RuleCodePrefix` and `RuleSelector`
    for (category, rules) in &category_to_rules {
        output.extend(super::rule_code_prefix::expand(
            category,
            rules
                .iter()
                .map(|(code, RuleMeta { group, attrs, .. })| (code.as_str(), group, attrs)),
        ));

        output.extend(quote! {
            impl From<#category> for RuleCodePrefix {
                fn from(category: #category) -> Self {
                    Self::#category(category)
                }
            }

            // Need ruff RuleSelector

            // // Rust doesn't yet support `impl const From<RuleCodePrefix> for RuleSelector`
            // // See https://github.com/rust-lang/rust/issues/67792
            // impl From<#category> for crate::rule_selector::RuleSelector {
            //     fn from(category: #category) -> Self {
            //         let prefix = RuleCodePrefix::#category(category);
            //         if is_single_rule_selector(&prefix) {
            //             Self::Rule {
            //                 prefix,
            //                 redirected_from: None,
            //             }
            //         } else {
            //             Self::Prefix {
            //                 prefix,
            //                 redirected_from: None,
            //             }
            //         }
            //     }
            // }
        });
    }

    // Create method that lists all rules for each individual `Category`
    for (category, rules) in &category_to_rules {
        let rules_by_prefix = rules_by_prefix(rules);

        let mut prefix_into_iter_match_arms = quote!();

        for (prefix, rules) in rules_by_prefix {
            let rule_paths = rules.iter().map(|(path, .., attrs)| {
                let rule_name = path.segments.last().unwrap();
                quote!(#(#attrs)* Rule::#rule_name)
            });
            let prefix_ident = get_prefix_ident(&prefix);
            let attrs = intersection_all(rules.iter().map(|(.., attrs)| attrs.as_slice()));
            let attrs = if attrs.is_empty() {
                quote!()
            } else {
                quote!(#(#attrs)*)
            };
            prefix_into_iter_match_arms.extend(quote! {
                #attrs #category::#prefix_ident => vec![#(#rule_paths,)*].into_iter(),
            });
        }

        output.extend(quote! {
            impl #category {
                pub fn rules(&self) -> ::std::vec::IntoIter<Rule> {
                    match self { #prefix_into_iter_match_arms }
                }
            }
        });
    }

    // Add methods for creating `RuleCodePrefix` and listing subcategory rules
    output.extend(quote! {
        impl RuleCodePrefix {
            pub fn parse(category: &Category, code: &str) -> Result<Self, crate::registry::FromCodeError> {
                use std::str::FromStr;

                Ok(match category {
                    #(Category::#category_idents => RuleCodePrefix::#category_idents(#category_idents::from_str(code).map_err(|_| crate::registry::FromCodeError::Unknown)?),)*
                })
            }

            pub fn rules(&self) -> ::std::vec::IntoIter<Rule> {
                match self {
                    #(RuleCodePrefix::#category_idents(prefix) => prefix.clone().rules(),)*
                }
            }
        }
    });

    let rule_to_code = generate_rule_to_code(&category_to_rules);
    output.extend(rule_to_code);

    output.extend(generate_iter_impl(&category_to_rules, &category_idents));

    Ok(output)
}

/// Group the rules by their common prefixes.
fn rules_by_prefix(
    rules: &BTreeMap<String, RuleMeta>,
) -> BTreeMap<String, Vec<(Path, Vec<Attribute>)>> {
    // TODO(charlie): Why do we do this here _and_ in `rule_code_prefix::expand`?
    let mut rules_by_prefix = BTreeMap::new();

    for code in rules.keys() {
        for i in 1..=code.len() {
            let prefix = code[..i].to_string();
            let rules: Vec<_> = rules
                .iter()
                .filter_map(|(code, rule)| {
                    if code.starts_with(&prefix) {
                        Some((rule.path.clone(), rule.attrs.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            rules_by_prefix.insert(prefix, rules);
        }
    }
    rules_by_prefix
}

/// Map from rule to codes that can be used to select it.
/// This abstraction exists to support a one-to-many mapping, whereby a single rule could map
/// to multiple codes (e.g., if it existed in multiple categorys, like Pylint and Flake8, under
/// different codes). We haven't actually activated this functionality yet, but some work was
/// done to support it, so the logic exists here.
fn generate_rule_to_code(
    category_to_rules: &BTreeMap<Ident, BTreeMap<String, RuleMeta>>,
) -> TokenStream {
    let mut rule_to_codes: HashMap<&Path, Vec<&RuleMeta>> = HashMap::new();
    let mut category_code_for_rule_match_arms = quote!();

    for (category, map) in category_to_rules {
        for (code, rule) in map {
            let RuleMeta {
                path, attrs, name, ..
            } = rule;
            rule_to_codes.entry(path).or_default().push(rule);
            category_code_for_rule_match_arms.extend(quote! {
                #(#attrs)* (Self::#category, Rule::#name) => Some(#code),
            });
        }
    }

    let mut rule_noqa_code_match_arms = quote!();
    let mut rule_group_match_arms = quote!();

    for (rule, codes) in rule_to_codes {
        let rule_name = rule.segments.last().unwrap();
        assert_eq!(
            codes.len(),
            1,
            "{} is mapped to multiple codes.",
            rule_name.ident
        );

        let RuleMeta {
            category,
            code,
            group,
            attrs,
            ..
        } = codes
            .iter()
            .sorted_by_key(|data| data.category == "Error")
            .next()
            .unwrap();

        rule_noqa_code_match_arms.extend(quote! {
            #(#attrs)* Rule::#rule_name => NoqaCode(crate::registry::Category::#category.common_prefix(), #code),
        });

        rule_group_match_arms.extend(quote! {
            #(#attrs)* Rule::#rule_name => #group,
        });
    }

    let rule_to_code = quote! {
        impl Rule {
            pub fn noqa_code(&self) -> NoqaCode {
                use crate::registry::RuleNamespace;

                match self {
                    #rule_noqa_code_match_arms
                }
            }

            pub fn group(&self) -> RuleGroup {
                use crate::registry::RuleNamespace;

                match self {
                    #rule_group_match_arms
                }
            }

            pub fn is_preview(&self) -> bool {
                matches!(self.group(), RuleGroup::Preview)
            }

            pub fn is_stable(&self) -> bool {
                matches!(self.group(), RuleGroup::Stable)
            }

            pub fn is_deprecated(&self) -> bool {
                matches!(self.group(), RuleGroup::Deprecated)
            }

            pub fn is_removed(&self) -> bool {
                matches!(self.group(), RuleGroup::Removed)
            }
        }

        impl Category {
            pub fn code_for_rule(&self, rule: Rule) -> Option<&'static str> {
                match (self, rule) {
                    #category_code_for_rule_match_arms
                    _ => None,
                }
            }
        }
    };
    rule_to_code
}

/// Implement `impl IntoIterator for &Category` and `RuleCodePrefix::iter()`
fn generate_iter_impl(
    category_to_rules: &BTreeMap<Ident, BTreeMap<String, RuleMeta>>,
    category_idents: &[&Ident],
) -> TokenStream {
    let mut category_rules_match_arms = quote!();
    let mut category_all_rules_match_arms = quote!();
    for (category, map) in category_to_rules {
        let rule_paths = map.values().map(|RuleMeta { attrs, path, .. }| {
            let rule_name = path.segments.last().unwrap();
            quote!(#(#attrs)* Rule::#rule_name)
        });
        category_rules_match_arms.extend(quote! {
            Category::#category => vec![#(#rule_paths,)*].into_iter(),
        });
        let rule_paths = map.values().map(|RuleMeta { attrs, path, .. }| {
            let rule_name = path.segments.last().unwrap();
            quote!(#(#attrs)* Rule::#rule_name)
        });
        category_all_rules_match_arms.extend(quote! {
            Category::#category => vec![#(#rule_paths,)*].into_iter(),
        });
    }

    quote! {
        impl Category {
            /// Rules not in the preview.
            pub fn rules(self: &Category) -> ::std::vec::IntoIter<Rule> {
                match self {
                    #category_rules_match_arms
                }
            }
            /// All rules, including those in the preview.
            pub fn all_rules(self: &Category) -> ::std::vec::IntoIter<Rule> {
                match self {
                    #category_all_rules_match_arms
                }
            }
        }

        impl RuleCodePrefix {
            pub fn iter() -> impl Iterator<Item = RuleCodePrefix> {
                use strum::IntoEnumIterator;

                let mut prefixes = Vec::new();

                #(prefixes.extend(#category_idents::iter().map(|x| Self::#category_idents(x)));)*
                prefixes.into_iter()
            }
        }
    }
}

/// Generate the `Rule` enum
fn register_rules<'a>(input: impl Iterator<Item = &'a RuleMeta>) -> TokenStream {
    let mut rule_variants = quote!();
    let mut rule_message_formats_match_arms = quote!();
    let mut rule_fixable_match_arms = quote!();
    let mut rule_explanation_match_arms = quote!();
    let mut rule_name_match_arms = quote!();

    let mut from_impls_for_diagnostic_kind = quote!();

    let mut path_rule_variants = quote!();
    let mut path_rule_from_match_arms = quote!();
    let mut path_rule_check_match_arms = quote!();

    let mut text_rule_variants = quote!();
    let mut text_rule_from_match_arms = quote!();
    let mut text_rule_check_match_arms = quote!();

    let mut ast_rule_variants = quote!();
    let mut ast_rule_from_match_arms = quote!();
    let mut ast_rule_check_match_arms = quote!();
    let mut ast_rule_entrypoint_match_arms = quote!();

    for RuleMeta {
        name,
        attrs,
        path,
        kind,
        ..
    } in input
    {
        rule_variants.extend(quote! {
            #(#attrs)*
            #name,
        });
        // Apply the `attrs` to each arm, like `[cfg(feature = "foo")]`.
        rule_message_formats_match_arms
            .extend(quote! {#(#attrs)* Self::#name => <#path as ruff_diagnostics::Violation>::message_formats(),});
        rule_fixable_match_arms.extend(
            quote! {#(#attrs)* Self::#name => <#path as ruff_diagnostics::Violation>::FIX_AVAILABILITY,},
        );
        rule_explanation_match_arms
            .extend(quote! {#(#attrs)* Self::#name => #path::explanation(),});
        rule_name_match_arms.extend(quote! {#(#attrs)* Self::#name => stringify!(#name),});

        // Enable conversion from `DiagnosticKind` to `Rule`.
        from_impls_for_diagnostic_kind
            .extend(quote! {#(#attrs)* stringify!(#name) => Rule::#name,});

        // Next parts are for creating two enums for the different
        // rule `check` signatures. This basically allows us to a)
        // partition a list of rules into the different check kinds
        // (path, text, ast), and b) call `rule.check(...)`. An
        // alternative might be to have different named check
        // functions (`check_text`, etc), and then partition based on
        // `rule.is_text()` or similar, but this way gives us some
        // type safety
        if kind.is_ident("Path") {
            path_rule_variants.extend(quote! {
                #(#attrs)*
                #name,
            });

            path_rule_from_match_arms.extend(quote! {
                #(#attrs)* Rule::#name => Ok(Self::#name),
            });

            path_rule_check_match_arms.extend(quote! {
                #(#attrs)* Self::#name => #path::check(settings, path),
            });
        }

        if kind.is_ident("Text") {
            text_rule_variants.extend(quote! {
                #(#attrs)*
                #name,
            });

            text_rule_from_match_arms.extend(quote! {
                #(#attrs)* Rule::#name => Ok(Self::#name),
            });

            text_rule_check_match_arms.extend(quote! {
                #(#attrs)* Self::#name => #path::check(settings, source),
            });
        }

        if kind.is_ident("Ast") {
            ast_rule_variants.extend(quote! {
                #(#attrs)*
                #name,
            });

            ast_rule_from_match_arms.extend(quote! {
                #(#attrs)* Rule::#name => Ok(Self::#name),
            });

            ast_rule_check_match_arms.extend(quote! {
                #(#attrs)* Self::#name => #path::check(settings, node, source),
            });

            ast_rule_entrypoint_match_arms.extend(quote! {
                #(#attrs)* Self::#name => #path::entrypoints(),
            });
        }
    }

    quote! {
        use std::path::Path;
        use ruff_diagnostics::{Diagnostic, Violation};
        use ruff_source_file::SourceFile;
        use tree_sitter::Node;
        use crate::{AstRule, PathRule, TextRule};
        use crate::settings::Settings;


        #[derive(
            Debug,
            PartialEq,
            Eq,
            Copy,
            Clone,
            Hash,
            PartialOrd,
            Ord,
            ::ruff_macros::CacheKey,
            ::strum_macros::AsRefStr,
            ::strum_macros::Display,
            ::strum_macros::EnumIter,
            ::strum_macros::EnumString,
            ::strum_macros::IntoStaticStr,
        )]
        #[repr(u16)]
        #[strum(serialize_all = "kebab-case")]
        pub enum Rule { #rule_variants }

        impl Rule {
            /// Returns the format strings used to report violations of this rule.
            pub fn message_formats(&self) -> &'static [&'static str] {
                match self { #rule_message_formats_match_arms }
            }

            /// Returns the documentation for this rule.
            pub fn explanation(&self) -> Option<&'static str> {
                match self { #rule_explanation_match_arms }
            }

            /// Returns the name for this rule.
            pub fn name(&self) -> &'static str {
                match self { #rule_name_match_arms }
            }

            /// Returns the fix status of this rule.
            pub const fn fixable(&self) -> ruff_diagnostics::FixAvailability {
                match self { #rule_fixable_match_arms }
            }

        }

        impl AsRule for ruff_diagnostics::DiagnosticKind {
            fn rule(&self) -> Rule {
                match self.name.as_str() {
                    #from_impls_for_diagnostic_kind
                    _ => unreachable!("invalid rule name: {}", self.name),
                }
            }
        }

        #[derive(
            Debug,
            PartialEq,
            Eq,
            Copy,
            Clone,
            Hash,
            PartialOrd,
            Ord,
            ::ruff_macros::CacheKey,
            ::strum_macros::AsRefStr,
            ::strum_macros::Display,
            ::strum_macros::EnumIter,
            ::strum_macros::EnumString,
            ::strum_macros::IntoStaticStr,
        )]
        #[repr(u16)]
        #[strum(serialize_all = "kebab-case")]
        pub enum PathRuleEnum { #path_rule_variants }

        impl TryFrom<Rule> for PathRuleEnum {
            type Error = &'static str;

            fn try_from(rule: Rule) -> Result<Self, Self::Error> {
                match rule {
                    #path_rule_from_match_arms
                    _ => Err("not a PathRule")
                }
            }
        }

        impl PathRuleEnum {
            pub fn check(&self, settings: &Settings, path: &Path) -> Option<Diagnostic> {
                match self {
                    #path_rule_check_match_arms
                }
            }
        }

        #[derive(
            Debug,
            PartialEq,
            Eq,
            Copy,
            Clone,
            Hash,
            PartialOrd,
            Ord,
            ::ruff_macros::CacheKey,
            ::strum_macros::AsRefStr,
            ::strum_macros::Display,
            ::strum_macros::EnumIter,
            ::strum_macros::EnumString,
            ::strum_macros::IntoStaticStr,
        )]
        #[repr(u16)]
        #[strum(serialize_all = "kebab-case")]
        pub enum TextRuleEnum { #text_rule_variants }

        impl TryFrom<Rule> for TextRuleEnum {
            type Error = &'static str;

            fn try_from(rule: Rule) -> Result<Self, Self::Error> {
                match rule {
                    #text_rule_from_match_arms
                    _ => Err("not a TextRule")
                }
            }
        }

        impl TextRuleEnum {
            pub fn check(&self, settings: &Settings, source: &SourceFile) -> Vec<Diagnostic> {
                match self {
                    #text_rule_check_match_arms
                }
            }
        }

        #[derive(
            Debug,
            PartialEq,
            Eq,
            Copy,
            Clone,
            Hash,
            PartialOrd,
            Ord,
            ::ruff_macros::CacheKey,
            ::strum_macros::AsRefStr,
            ::strum_macros::Display,
            ::strum_macros::EnumIter,
            ::strum_macros::EnumString,
            ::strum_macros::IntoStaticStr,
        )]
        #[repr(u16)]
        #[strum(serialize_all = "kebab-case")]
        pub enum AstRuleEnum { #ast_rule_variants }

        impl TryFrom<Rule> for AstRuleEnum {
            type Error = &'static str;

            fn try_from(rule: Rule) -> Result<Self, Self::Error> {
                match rule {
                    #ast_rule_from_match_arms
                    _ => Err("not an AstRule")
                }
            }
        }

        impl AstRuleEnum {
            pub fn check(&self, settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
                match self {
                    #ast_rule_check_match_arms
                }
            }

            pub fn entrypoints(&self) -> Vec<&'static str> {
                match self {
                    #ast_rule_entrypoint_match_arms
                }
            }
        }
    }
}

impl Parse for RuleMeta {
    /// Parses a match arm such as `(Pycodestyle, "E112") => (RuleGroup::Preview, rules::pycodestyle::rules::logical_lines::NoIndentedBlock),`
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let pat_tuple;
        parenthesized!(pat_tuple in input);
        let category: Ident = pat_tuple.parse()?;
        let _: Token!(,) = pat_tuple.parse()?;
        let code: LitStr = pat_tuple.parse()?;
        let _: Token!(=>) = input.parse()?;
        let pat_tuple;
        parenthesized!(pat_tuple in input);
        let group: Path = pat_tuple.parse()?;
        let _: Token!(,) = pat_tuple.parse()?;
        let kind: Path = pat_tuple.parse()?;

        let kind_is_valid = kind.is_ident("Path")
            || kind.is_ident("Text")
            || kind.is_ident("Ast")
            || kind.is_ident("Test");
        if !kind_is_valid {
            // We better have an ident here, because I don't know what else to do
            let kind = kind.get_ident().unwrap();
            return Err(syn::Error::new(
                pat_tuple.span(),
                format!(
                    "Invalid checker kind '{kind}', expected one of 'Path', 'Text', 'Ast', 'Test'"
                ),
            ));
        }

        let _: Token!(,) = pat_tuple.parse()?;
        let rule_path: Path = pat_tuple.parse()?;
        let _: Token!(,) = input.parse()?;
        let rule_name = rule_path.segments.last().unwrap().ident.clone();
        Ok(RuleMeta {
            name: rule_name,
            category,
            code,
            group,
            kind,
            path: rule_path,
            attrs,
        })
    }
}
