macro_rules! num_ast {
    (AST) => {1};
    ($type: tt) => {0};
    (AST, $($types: tt), +) => {
        1 + num_ast!($($types), +)
    };
    ($type: tt, $($types: tt), +) => {
        num_ast!($($types), +)
    }
}

macro_rules! option_ast {
    (AST, $value: tt) => {
        Some($value)
    };
    ($type: tt, $value: tt) => {
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

        const N_AST: usize = num_ast!($($types), +);
        const AST_OPTIONS: &[Option<&str>; CODES.len()] = &[$(option_ast!($types, $codes)), +];
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
