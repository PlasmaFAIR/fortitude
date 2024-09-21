macro_rules! num_rules {
    ($rule: ident) => {
        1
    };
    ($rule: path, $($rules: ident), +) => {
        1 + num_rules!($($rules), +)
    };
}

#[macro_export]
macro_rules! register_rules {
    ($(($categories: ty, $codes: literal, $paths: path, $rules: ident)), +) => {

        const N_RULES: usize = num_rules!($($rules),+);

        const CODES: &[&str; N_RULES] = &[$($codes,) +];

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
