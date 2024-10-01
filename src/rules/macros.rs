macro_rules! num_types {
    (PATH, PATH) => {1};
    (TEXT, TEXT) => {1};
    (AST, AST) => {1};
    ($type1: tt, $type2: tt) => {0};
    (PATH, PATH, $($types: tt), +) => {
        1 + num_types!(PATH, $($types), +)
    };
    (TEXT, TEXT, $($types: tt), +) => {
        1 + num_types!(TEXT, $($types), +)
    };
    (AST, AST, $($types: tt), +) => {
        1 + num_types!(AST, $($types), +)
    };
    ($type1: tt, $type2: tt, $($types: tt), +) => {
        num_types!($type1, $($types), +)
    };
}

macro_rules! some_type {
    (PATH, PATH, $value: tt) => {
        Some($value)
    };
    (AST, AST, $value: tt) => {
        Some($value)
    };
    (TEXT, TEXT, $value: tt) => {
        Some($value)
    };
    ($type1: tt, $type2: tt, $value: tt) => {
        None
    };
}

macro_rules! build_set {
    ($name: ident, $array: ident) => {
        lazy_static! {
            static ref $name: BTreeSet<&'static str> = { BTreeSet::from(*$array) };
        }
    };
}

macro_rules! add_rule_to_map {
    (PATH, PATH, $code: literal, $rule: ident, $map: ident, $settings: ident) => {
        $map.insert($code, Box::new($rule::new($settings)));
    };
    (TEXT, TEXT, $code: literal, $rule: ident, $map: ident, $settings: ident) => {
        $map.insert($code, Box::new($rule::new($settings)));
    };
    (AST, AST, $code: literal, $rule: ident, $map: ident, $settings: ident) => {
        let rule = $rule::new($settings);
        for entrypoint in rule.entrypoints() {
            match $map.get_mut(entrypoint) {
                Some(rule_vec) => {
                    rule_vec.push(($code, Box::new($rule::new($settings))));
                }
                None => {
                    $map.insert(entrypoint, vec![($code, Box::new($rule::new($settings)))]);
                }
            }
        }
    };
    ($type1: tt, $type2: tt, $code: literal, $rule: ident, $map: ident, $settings: ident) => {
        ();
    };
}

#[macro_export]
macro_rules! register_rules {
    ($(($categories: ty, $codes: literal, $types: tt, $paths: path, $rules: ident)), +) => {

        use lazy_static::lazy_static;
        use std::collections::{BTreeSet, BTreeMap};
        use $crate::{Rule, ASTRule, PathRule, TextRule};
        use $crate::settings::Settings;

        const _CODES: &[&'static str; [$($codes), *].len()] = &[$($codes), +];
        build_set!(CODES, _CODES);

        const fn drop_none<const N1: usize, const N2: usize>(xs: &[Option<&'static str>; N1]) -> [&'static str; N2] {
            let mut result: [&'static str; N2] = [""; N2];
            let mut i = 0;
            let mut j = 0;
            while i < N1 {
                if let Some(y) = xs[i] {
                    result[j] = y;
                    j += 1;
                }
                i += 1;
            }
            result
        }

        const _N_PATH: usize = num_types!(PATH, $($types), +);
        const _PATH_OPTIONS: &[Option<&str>; _CODES.len()] = &[$(some_type!(PATH, $types, $codes)), +];
        const _PATH_CODES: &[&str; _N_PATH] = &drop_none(_PATH_OPTIONS);
        build_set!(PATH_CODES, _PATH_CODES);

        const _N_TEXT: usize = num_types!(TEXT, $($types), +);
        const _TEXT_OPTIONS: &[Option<&str>; _CODES.len()] = &[$(some_type!(TEXT, $types, $codes)), +];
        const _TEXT_CODES: &[&str; _N_TEXT] = &drop_none(_TEXT_OPTIONS);
        build_set!(TEXT_CODES, _TEXT_CODES);

        const _N_AST: usize = num_types!(AST, $($types), +);
        const _AST_OPTIONS: &[Option<&str>; _CODES.len()] = &[$(some_type!(AST, $types, $codes)), +];
        const _AST_CODES: &[&str; _N_AST] = &drop_none(_AST_OPTIONS);
        build_set!(AST_CODES, _AST_CODES);

        // Make a local type alias to each rule.
        // Needed as macro rules forbids calling ::new() on type paths.
        $(type $rules = $paths;)+

        pub type RuleSet<'a> = BTreeSet<&'a str>;
        pub type PathRuleMap<'a> = BTreeMap<&'a str, Box<dyn PathRule>>;
        pub type TextRuleMap<'a> = BTreeMap<&'a str, Box<dyn TextRule>>;
        pub type ASTEntryPointMap<'a> = BTreeMap<&'a str, Vec<(&'a str, Box<dyn ASTRule>)>>;

        // Returns the full set of all rules.
        pub fn full_ruleset<'a>() -> RuleSet<'a> {
            CODES.clone()
        }

        // Returns the set of rules that are activated by default, expressed as strings.
        pub fn default_ruleset<'a>() -> RuleSet<'a> {
            // Currently all rules are activated by default.
            // Should add an additional macro input to toggle default or not.
            // Community feedback will be needed to determine a sensible set.
            full_ruleset()
        }

        // Create a mapping of codes to rule instances that operate on paths.
        pub fn path_rule_map<'a>(codes: &'a RuleSet<'a>, settings: &'a Settings) -> PathRuleMap<'a> {
            let path_codes: RuleSet = PATH_CODES.intersection(&codes).copied().collect();
            let mut map = PathRuleMap::new();
            for code in path_codes {
                match code {
                    $($codes => {add_rule_to_map!(PATH, $types, $codes, $rules, map, settings);})+
                    _ => {
                        continue;
                    }
                }
            }
            map
        }

        // Create a mapping of codes to rule instances that operate on lines of code directly.
        pub fn text_rule_map<'a>(codes: &'a RuleSet<'a>, settings: &'a Settings) -> TextRuleMap<'a> {
            let text_codes: RuleSet = TEXT_CODES.intersection(&codes).copied().collect();
            let mut map = TextRuleMap::new();
            for code in text_codes {
                match code {
                    $($codes => {add_rule_to_map!(TEXT, $types, $codes, $rules, map, settings);})+
                    _ => {
                        continue;
                    }
                }
            }
            map
        }

        // Create a mapping of AST entrypoints to lists of the rules and codes that operate on them.
        pub fn ast_entrypoint_map<'a>(codes: &'a RuleSet<'a>, settings: &'a Settings) -> ASTEntryPointMap<'a> {
            let ast_codes: RuleSet = AST_CODES.intersection(&codes).copied().collect();
            let mut map = ASTEntryPointMap::new();
            for code in ast_codes {
                match code {
                    $($codes => {add_rule_to_map!(AST, $types, $codes, $rules, map, settings);})+
                    _ => {
                        continue;
                    }
                }
            }
            map
        }

        // Print the help text for a rule.
        pub fn explain_rule<'a>(code: &'a str, settings: &'a Settings) -> &'a str {
            match code {
                $($codes => {$rules::new(&settings).explain()})+
                _ => {
                    ""
                }
            }
        }
    };
}

/// Shorthand for `Some(vec![x])`. Note that an empty argument list
/// is forbidden -- functions should just return `None` instead
#[macro_export]
macro_rules! some_vec {
    ($($x:expr),+$(,)?) => {
        Some(vec![$($x),+])
    };
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    #[test]
    fn test_some_vec() -> anyhow::Result<()> {
        let expected = Some(vec![1, 2, 3]);
        let actual = some_vec![1, 2, 3];

        assert_eq!(expected, actual);
        Ok(())
    }
}
