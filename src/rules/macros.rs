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

#[macro_export]
macro_rules! register_rules {
    ($(($categories: ty, $codes: literal, $types: tt, $paths: path, $rules: ident)), +) => {

        const CODES: &[&str; [$($codes), *].len()] = &[$($codes), +];

        const fn drop_none<'a, const N1: usize, const N2: usize>(xs: &'a [Option<&'a str>; N1]) -> [&'a str; N2] {
            let mut result: [&str; N2] = [""; N2];
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

        const N_PATH: usize = num_types!(PATH, $($types), +);
        const PATH_OPTIONS: &[Option<&str>; CODES.len()] = &[$(some_type!(PATH, $types, $codes)), +];
        const PATH_CODES: &[&str; N_PATH] = &drop_none(PATH_OPTIONS);

        const N_TEXT: usize = num_types!(TEXT, $($types), +);
        const TEXT_OPTIONS: &[Option<&str>; CODES.len()] = &[$(some_type!(TEXT, $types, $codes)), +];
        const TEXT_CODES: &[&str; N_TEXT] = &drop_none(TEXT_OPTIONS);

        const N_AST: usize = num_types!(AST, $($types), +);
        const AST_OPTIONS: &[Option<&str>; CODES.len()] = &[$(some_type!(AST, $types, $codes)), +];
        const AST_CODES: &[&str; N_AST] = &drop_none(AST_OPTIONS);

        $(type $rules = $paths;)+

        /// Create a new `Rule` given a rule code, expressed as a string.
        pub fn build_rule(code: &str) -> anyhow::Result<Box<dyn Rule>> {
            match code {
                $($codes => Ok(Box::new($rules {})),)+
                _ => {
                    anyhow::bail!("Unknown rule code {}", code)
                }
            }
        }
    };
}
