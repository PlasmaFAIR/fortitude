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
