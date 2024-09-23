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
    ($type1: tt, $type2: tt, $code: literal, $rule: ident, $map: ident, $settings: ident) => {
        ();
    };
}

macro_rules! match_result {
    (PATH, $code: literal, $rule: ident) => {
        anyhow::bail!("Unknown rule code {}", $code)
    };
    (TEXT, $code: literal, $rule: ident) => {
        anyhow::bail!("Unknown rule code {}", $code)
    };
    ($type: tt, $code: literal, $rule: ident) => {
        Ok(Box::new($rule {}))
    };
}

#[macro_export]
macro_rules! register_rules {
    ($(($categories: ty, $codes: literal, $types: tt, $paths: path, $rules: ident)), +) => {

        use lazy_static::lazy_static;
        use std::collections::BTreeSet;
        use $crate::BaseRule;
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
        $(type $rules = $paths;)+

        // Create a new `Rule` given a rule code, expressed as a string.
        pub fn build_rule(code: &str) -> anyhow::Result<Box<dyn Rule>> {
            match code {
                $($codes => match_result!($types, $codes, $rules),)+
                _ => {
                    anyhow::bail!("Unknown rule code {}", code)
                }
            }
        }

        pub type PathRuleMap<'a>  = BTreeMap<&'a str, Box<dyn PathRule>>;
        pub type TextRuleMap<'a>  = BTreeMap<&'a str, Box<dyn TextRule>>;

        // Create a mapping of codes to rule instances that operate on paths.
        pub fn path_rule_map<'a>(codes: &'a BTreeSet<&'a str>, settings: &'a Settings) -> PathRuleMap<'a> {
            let path_codes: BTreeSet<_> = PATH_CODES.intersection(&codes).collect();
            let mut map = PathRuleMap::new();
            for code in path_codes {
                match *code {
                    $($codes => {add_rule_to_map!(PATH, $types, $codes, $rules, map, settings);})+
                    _ => {
                        continue;
                    }
                }
            }
            map
        }

        // Create a mapping of codes to rule instances that operate on lines of code directly.
        pub fn text_rule_map<'a>(codes: &'a BTreeSet<&'a str>, settings: &'a Settings) -> TextRuleMap<'a> {
            let text_codes: BTreeSet<_> = TEXT_CODES.intersection(&codes).collect();
            let mut map = TextRuleMap::new();
            for code in text_codes {
                match *code {
                    $($codes => {add_rule_to_map!(TEXT, $types, $codes, $rules, map, settings);})+
                    _ => {
                        continue;
                    }
                }
            }
            map
        }
    };
}
